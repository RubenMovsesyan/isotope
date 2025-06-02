use std::{
    borrow::Cow,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use anyhow::{Result, anyhow};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor,
    PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{element::buffered::Buffered, gpu_utils::GpuController};

#[derive(Debug, Default)]
pub struct ParallelInstancerBuilder<'a> {
    compute_shader: Option<Cow<'a, str>>,
    label: Option<String>,
    bind_group_layouts: Option<Vec<BindGroupLayout>>,
    bind_groups: Option<Vec<BindGroup>>,
    instance_count: Option<u64>,
    pipeline_compilaiton_options: Option<PipelineCompilationOptions<'a>>,
}

impl<'a> ParallelInstancerBuilder<'a> {
    pub fn with_compute_shader(&mut self, compute_shader_code: &'a str) -> &mut Self {
        self.compute_shader = Some(Cow::Borrowed(compute_shader_code));

        self
    }

    pub fn with_label<S>(&mut self, label: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.label = Some(label.as_ref().to_string());

        self
    }

    pub fn with_instance_count(&mut self, instance_count: u64) -> &mut Self {
        self.instance_count = Some(instance_count);

        self
    }

    /// Add Bind Group Layouts
    pub fn add_bind_group_layout(&mut self, bind_group_layout: BindGroupLayout) -> &mut Self {
        if let Some(bind_group_layouts) = self.bind_group_layouts.as_mut() {
            bind_group_layouts.push(bind_group_layout);
        } else {
            self.bind_group_layouts = Some(Vec::from([bind_group_layout]));
        }

        self
    }

    /// Add Bind Group
    pub fn add_bind_group(&mut self, bind_group: BindGroup) -> &mut Self {
        if let Some(bind_groups) = self.bind_groups.as_mut() {
            bind_groups.push(bind_group);
        } else {
            self.bind_groups = Some(Vec::from([bind_group]))
        }

        self
    }

    /// Add Bind Group Layout and Bind Group
    pub fn add_bind_group_with_layout(
        &mut self,
        bind_group_with_layout: (BindGroupLayout, BindGroup),
    ) -> &mut Self {
        self.add_bind_group_layout(bind_group_with_layout.0)
            .add_bind_group(bind_group_with_layout.1)
    }

    // helper function to copy the label from the builder
    fn get_label_with(&self, connecting_label: &str) -> String {
        let label = match self.label.as_ref() {
            Some(label) => label.clone() + " ",
            None => "".to_string(),
        };

        label + connecting_label
    }

    pub(crate) fn build<T: Instance>(
        &mut self,
        gpu_controller: Arc<GpuController>,
    ) -> Result<Instancer<T>> {
        let instance_count = self
            .instance_count
            .take()
            .ok_or(anyhow!("Instance Count Not Set"))?;

        let mut bind_group_layouts = self.bind_group_layouts.take().unwrap_or_else(|| Vec::new());
        bind_group_layouts.push(gpu_controller.device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("Instancer Instance Buffer Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        ));

        // let instance_buffer = gpu_controller.device.create_buffer(&BufferDescriptor {
        //     label: Some(&self.get_label_with("Instance Buffer")),
        //     mapped_at_creation: false,
        //     size: instance_count * std::mem::size_of::<T>() as u64,
        //     usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        // });
        let instance_buffer = gpu_controller
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(&self.get_label_with("Instance Buffer")),
                contents: bytemuck::cast_slice(
                    &(0..instance_count)
                        .into_iter()
                        .map(|_| T::default())
                        .collect::<Vec<T>>(),
                ),
                usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

        let bind_groups = self.bind_groups.take().unwrap_or_else(|| Vec::new());
        let instance_buffer_bind_group =
            gpu_controller
                .device
                .create_bind_group(&BindGroupDescriptor {
                    label: Some("Instancer Instance Buffer Bind Group"),
                    layout: unsafe { bind_group_layouts.last().unwrap_unchecked() },
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: instance_buffer.as_entire_binding(),
                    }],
                });

        let compute_shader = gpu_controller
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some(&self.get_label_with("Compute Shader")),
                source: ShaderSource::Wgsl(
                    self.compute_shader
                        .take()
                        .ok_or(anyhow!("Compute Shader Not Set"))?,
                ),
            });

        let compute_pipeline = {
            let layout = gpu_controller
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&self.get_label_with("Compute Pipeline Layout")),
                    bind_group_layouts: bind_group_layouts
                        .iter()
                        .map(|bgl| bgl)
                        .collect::<Vec<&BindGroupLayout>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });

            gpu_controller
                .device
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some(&self.get_label_with("Compute Pipeline")),
                    layout: Some(&layout),
                    entry_point: Some("main"),
                    module: &compute_shader,
                    compilation_options: match self.pipeline_compilaiton_options.take() {
                        Some(options) => options,
                        None => PipelineCompilationOptions::default(),
                    },
                    cache: None,
                })
        };

        Ok(Instancer {
            label: self.get_label_with(""),
            gpu_controller,
            instance_count,
            instance_buffer,
            instance_buffer_bind_group,
            instancer_type: InstancerType::Parallel {
                compute_pipeline,
                bind_groups,
            },
        })
    }
}

/// Implementers must mark their type with #[repr(C)] to ensure consistent memory layout
pub(crate) unsafe trait Instance:
    Debug + Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable + Buffered
{
}

#[derive(Debug)]
enum InstancerType<T: Instance> {
    Series {
        instances: RwLock<Vec<T>>,
    },
    Parallel {
        compute_pipeline: ComputePipeline,
        bind_groups: Vec<BindGroup>,
    },
}

#[derive(Debug)]
pub(crate) struct Instancer<T: Instance> {
    gpu_controller: Arc<GpuController>,
    pub(crate) instance_count: u64,
    pub(crate) instance_buffer: Buffer,
    instance_buffer_bind_group: BindGroup,
    label: String,

    instancer_type: InstancerType<T>,
}

#[derive(Debug)]
pub(crate) enum InstanceBufferDescriptor<T: Instance> {
    Size(u64),
    Instances(Vec<T>),
}

impl<T: Instance> Instancer<T> {
    pub fn new_series<S>(
        gpu_controller: Arc<GpuController>,
        buffer_descriptor: InstanceBufferDescriptor<T>,
        label: S,
    ) -> Self
    where
        S: AsRef<str>,
    {
        let (instance_buffer, instance_count, instancer_type) = match buffer_descriptor {
            InstanceBufferDescriptor::Size(buffer_size) => {
                let instance_buffer =
                    gpu_controller
                        .device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&[T::default()]),
                            usage: BufferUsages::VERTEX
                                | BufferUsages::STORAGE
                                | BufferUsages::COPY_DST,
                        });

                // Create a series instancer with enough instances to fill the buffer
                let instancer_type = InstancerType::Series {
                    instances: RwLock::new(
                        (0..buffer_size).into_iter().map(|_| T::default()).collect(),
                    ),
                };

                (instance_buffer, buffer_size, instancer_type)
            }
            InstanceBufferDescriptor::Instances(instances) => {
                let instance_buffer =
                    gpu_controller
                        .device
                        .create_buffer_init(&BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&instances),
                            usage: BufferUsages::VERTEX
                                | BufferUsages::STORAGE
                                | BufferUsages::COPY_DST,
                        });

                let instance_count = instances.len() as u64;

                let instancer_type = InstancerType::Series {
                    instances: RwLock::new(instances),
                };

                (instance_buffer, instance_count, instancer_type)
            }
        };

        let instance_buffer_bind_group_layout =
            gpu_controller
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Instancer Instance Buffer Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let instance_buffer_bind_group =
            gpu_controller
                .device
                .create_bind_group(&BindGroupDescriptor {
                    label: Some("Instancer Instance Buffer Bind Group Layout"),
                    layout: &instance_buffer_bind_group_layout,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: instance_buffer.as_entire_binding(),
                    }],
                });

        Self {
            label: label.as_ref().to_string(),
            gpu_controller,
            instance_buffer,
            instance_count,
            instance_buffer_bind_group,
            instancer_type,
        }
    }

    pub fn compute_instances<F>(&self, callback: F)
    where
        F: FnOnce(&mut [T]),
    {
        match &self.instancer_type {
            InstancerType::Series { instances } => {
                if let Ok(mut instances) = instances.write() {
                    callback(&mut instances);
                }
            }
            InstancerType::Parallel {
                compute_pipeline,
                bind_groups,
                ..
            } => {
                let mut encoder =
                    self.gpu_controller
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Instancer Command encoder"),
                        });

                {
                    // Temp for now
                    let dispatch_size = 256;

                    // Begin the compute pass
                    let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                        label: Some(&self.label),
                        timestamp_writes: None,
                    });

                    // Set the pipeline
                    compute_pass.set_pipeline(compute_pipeline);

                    // Set all custom bind groups
                    for (index, bind_group) in bind_groups.iter().enumerate() {
                        compute_pass.set_bind_group(index as u32, bind_group, &[]);
                    }

                    // Set the instance buffer bind group at the end
                    compute_pass.set_bind_group(
                        bind_groups.len() as u32,
                        &self.instance_buffer_bind_group,
                        &[],
                    );

                    compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
                }

                self.gpu_controller.queue.submit(Some(encoder.finish()));
            }
        }
    }
}
