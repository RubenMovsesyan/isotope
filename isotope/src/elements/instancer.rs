use std::{ops::Range, time::Instant};

use anyhow::{Result, anyhow};
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    ComputePipeline, ComputePipelineDescriptor, GpuController, Instance,
    PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModule, ShaderStages,
};
use isotope_utils::ToHash;

use crate::AssetServer;

pub trait PodData: Send + Sync {
    fn as_bytes(&self) -> &[u8];
    fn len(&self) -> usize;
}

impl<T: bytemuck::Pod + Send + Sync> PodData for Vec<T> {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }

    fn len(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }
}

pub enum InstancerBinding {
    Uniform(Box<dyn PodData>),
    StorageRO(Box<dyn PodData>),
    StorageRW(Box<dyn PodData>),
}

impl InstancerBinding {
    pub fn new_uniform<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::Uniform(Box::new(data))
    }

    pub fn new_storage_ro<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::StorageRO(Box::new(data))
    }

    pub fn new_storage_rw<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::StorageRW(Box::new(data))
    }

    fn data(&self) -> &dyn PodData {
        match self {
            InstancerBinding::Uniform(data) => data.as_ref(),
            InstancerBinding::StorageRO(data) => data.as_ref(),
            InstancerBinding::StorageRW(data) => data.as_ref(),
        }
    }

    fn binding_type(&self) -> BufferBindingType {
        match self {
            InstancerBinding::Uniform(_) => BufferBindingType::Uniform,
            InstancerBinding::StorageRO(_) => BufferBindingType::Storage { read_only: true },
            InstancerBinding::StorageRW(_) => BufferBindingType::Storage { read_only: false },
        }
    }

    fn buffer_usages(&self) -> BufferUsages {
        match self {
            InstancerBinding::Uniform(_) => BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            _ => BufferUsages::STORAGE | BufferUsages::COPY_DST,
        }
    }
}

pub type SerialModifier = fn(&mut [Instance], f32, f32);

pub struct Instancer {
    pub(crate) range: Option<Range<u64>>,
    pub(crate) instancer_kind: InstancerKind,
}

pub(crate) enum InstancerKind {
    Serial {
        serial_modifier: SerialModifier,
    },
    Parallel {
        shader_hash: String,
        shader: ShaderModule,
        pipeline: ComputePipeline,
        bind_group: Option<BindGroup>,
        buffers: Vec<Buffer>,
    },
}

impl Instancer {
    pub fn new_serial(range: Option<Range<u64>>, serial_modifier: SerialModifier) -> Self {
        Self {
            range,
            instancer_kind: InstancerKind::Serial { serial_modifier },
        }
    }

    pub fn new_parallel(
        range: Option<Range<u64>>,
        asset_server: &AssetServer,
        bindings: Vec<InstancerBinding>,
        shader: &str,
    ) -> Result<Self> {
        let shader_module = asset_server.gpu_controller.create_shader(shader);
        let shader_hash = shader.to_hash();

        let mut buffers: Vec<Buffer> = Vec::new();

        // Create the delta_t and t buffers first
        for i in 0..2 {
            buffers.push(
                asset_server
                    .gpu_controller
                    .create_buffer(&BufferDescriptor {
                        label: Some(&format!(
                            "Instancer Buffer Binding: {}",
                            if i == 0 { "delta_t" } else { "t" }
                        )),
                        mapped_at_creation: false,
                        size: std::mem::size_of::<f32>() as u64,
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    }),
            );
        }

        // Create the buffers first
        for (binding_count, binding) in bindings.iter().enumerate() {
            buffers.push(
                asset_server
                    .gpu_controller
                    .create_buffer(&BufferDescriptor {
                        label: Some(&format!("Instancer Buffer Binding: {}", binding_count + 3)),
                        mapped_at_creation: false,
                        size: binding.data().len() as u64,
                        usage: binding.buffer_usages(),
                    }),
            );
        }

        let mut bind_group_layout_entries: Vec<BindGroupLayoutEntry> = Vec::new();

        // Create the bindings based on the types of storage that is passed into the function
        bind_group_layout_entries.extend_from_slice(&[
            // Instance Buffer
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            // Delta t
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            // Time
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ]);

        let mut binding_count = 1; // The first bindings is going to be the instance buffer and time buffers
        bindings.iter().for_each(|binding| {
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_count,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: binding.binding_type(),
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            });

            binding_count += 1;
        });

        // Add the Bind group layout to the layouts manager
        asset_server.gpu_controller.write_layouts(|layouts| {
            layouts.insert(
                shader_hash.clone(),
                asset_server
                    .gpu_controller
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("Instancer"),
                        entries: &bind_group_layout_entries,
                    }),
            );
        })?;

        // Create the pipeline based on the new layout
        let pipeline =
            asset_server
                .gpu_controller
                .read_layouts(|layouts| -> Result<ComputePipeline> {
                    let bind_group_layout = if let Some(layout) = layouts.get(&shader_hash) {
                        Ok(layout)
                    } else {
                        Err(anyhow!(
                            "Failed To get Bind Group Layout with hash: {}",
                            shader_hash
                        ))
                    }?;

                    let pipeline_layout = asset_server.gpu_controller.create_pipeline_layout(
                        &PipelineLayoutDescriptor {
                            label: Some("Instancer Pipeline Layout"),
                            bind_group_layouts: &[bind_group_layout],
                            push_constant_ranges: &[],
                        },
                    );

                    Ok(asset_server.gpu_controller.create_compute_pipeline(
                        &ComputePipelineDescriptor {
                            label: Some("Instancer"),
                            cache: None,
                            compilation_options: PipelineCompilationOptions::default(),
                            entry_point: Some("main"),
                            layout: Some(&pipeline_layout),
                            module: &shader_module,
                        },
                    ))
                })??;

        Ok(Self {
            range,
            instancer_kind: InstancerKind::Parallel {
                shader_hash,
                shader: shader_module,
                pipeline,
                bind_group: None,
                buffers,
            },
        })
    }

    pub(crate) fn prepare_for_instancing(
        &mut self,
        instance_buffer: &Buffer,
        gpu_controller: &GpuController,
        dt: f32,
        t: f32,
    ) -> Option<&BindGroup> {
        match &mut self.instancer_kind {
            InstancerKind::Parallel {
                shader_hash,
                bind_group,
                buffers,
                ..
            } => {
                // Write the dt and t values to the buffers
                gpu_controller.write_buffer(&buffers[0], 0, bytemuck::cast_slice(&[dt]));
                gpu_controller.write_buffer(&buffers[1], 0, bytemuck::cast_slice(&[t]));

                if bind_group.is_none() {
                    let mut bind_group_entries: Vec<BindGroupEntry> = Vec::new();
                    bind_group_entries.push(BindGroupEntry {
                        binding: 0,
                        resource: instance_buffer.as_entire_binding(),
                    });

                    for (index, buffer) in buffers.iter().enumerate() {
                        bind_group_entries.push(BindGroupEntry {
                            binding: index as u32 + 1,
                            resource: buffer.as_entire_binding(),
                        });
                    }

                    // TODO: Clean this up
                    *bind_group = gpu_controller
                        .read_layouts(|layouts| {
                            let bind_group_layout =
                                layouts.get(&shader_hash.clone()).expect(&format!(
                                    "Failed To Get Bind Group Layout with hash: {}",
                                    shader_hash
                                ));

                            Some(gpu_controller.create_bind_group(&BindGroupDescriptor {
                                label: Some("GPU Instancer Bind Group"),
                                layout: bind_group_layout,
                                entries: &bind_group_entries,
                            }))
                        })
                        .expect("Failed To Create Bind Group");
                }

                bind_group.as_ref()
            }
            _ => None,
        }
    }
}
