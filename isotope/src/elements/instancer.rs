use std::ops::Range;

use anyhow::{Result, anyhow};
use gpu_controller::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor,
    BufferInitDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor, GpuController,
    Instance, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderStages,
};
use isotope_utils::ToHash;
use log::error;

use crate::AssetServer;

const BUILTING_BUFFERS: usize = 4;

/// A trait for data that can be used as GPU buffer contents.
///
/// This trait abstracts over different types of data that can be converted
/// to byte slices for GPU buffer operations. The data must be thread-safe
/// (Send + Sync) to work with the GPU controller's threading model.
pub trait PodData: Send + Sync {
    /// Returns a byte slice representation of the data.
    ///
    /// This method provides access to the raw bytes of the data structure
    /// for copying to GPU buffers.
    ///
    /// # Returns
    /// A byte slice (`&[u8]`) containing the raw representation of the data.
    fn as_bytes(&self) -> &[u8];

    /// Returns the size of the data in bytes.
    ///
    /// This method returns the total byte size of the data, which is used
    /// for buffer allocation and validation.
    ///
    /// # Returns
    /// The size of the data in bytes as a `usize`.
    fn len(&self) -> usize;
}

impl<T: bytemuck::Pod + Send + Sync> PodData for Vec<T> {
    /// Converts the vector to a byte slice using bytemuck for safe casting.
    ///
    /// This implementation uses bytemuck's `cast_slice` to safely convert
    /// a slice of Pod types to a byte slice. This is safe because Pod types
    /// have no invalid bit patterns and can be safely transmitted as raw bytes.
    ///
    /// # Returns
    /// A byte slice representation of the vector's contents.
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }

    /// Returns the total byte size of all elements in the vector.
    ///
    /// Calculates the size by multiplying the number of elements by the
    /// size of each element type.
    ///
    /// # Returns
    /// The total byte size of the vector's contents.
    fn len(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
    }
}

/// Represents different types of GPU buffer bindings for compute shaders.
///
/// This enum wraps different kinds of data that can be bound to compute shaders,
/// each with different access patterns and GPU buffer types:
/// - Uniform: Small, read-only data accessed efficiently by all shader invocations
/// - StorageRO: Large, read-only data accessed by individual shader invocations
/// - StorageRW: Large, read-write data that can be modified by shader invocations
pub enum InstancerBinding {
    /// A uniform buffer binding for small, frequently accessed read-only data.
    /// Uniform buffers have size limitations but provide fast access patterns.
    Uniform(Box<dyn PodData>),

    /// A read-only storage buffer binding for large amounts of read-only data.
    /// Storage buffers can be much larger than uniform buffers.
    StorageRO(Box<dyn PodData>),

    /// A read-write storage buffer binding for data that shaders can modify.
    /// Allows shaders to both read from and write to the buffer.
    StorageRW(Box<dyn PodData>),
}

impl InstancerBinding {
    /// Creates a new uniform buffer binding from a vector of Pod data.
    ///
    /// Uniform buffers are ideal for small amounts of data (typically < 64KB)
    /// that are accessed frequently by all shader invocations. They provide
    /// the fastest access pattern but have size limitations.
    ///
    /// # Type Parameters
    /// - `T`: The data type, must implement `bytemuck::Pod + Send + Sync`
    ///
    /// # Parameters
    /// - `data`: A vector containing the data to bind as a uniform buffer
    ///
    /// # Returns
    /// A new `InstancerBinding::Uniform` containing the provided data.
    ///
    /// # Example
    /// ```rust
    /// let matrices = vec![Matrix4::identity(); 10];
    /// let binding = InstancerBinding::new_uniform(matrices);
    /// ```
    pub fn new_uniform<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::Uniform(Box::new(data))
    }

    /// Creates a new read-only storage buffer binding from a vector of Pod data.
    ///
    /// Read-only storage buffers can hold much larger amounts of data than uniform
    /// buffers and are ideal for large datasets that shaders need to read from
    /// but not modify.
    ///
    /// # Type Parameters
    /// - `T`: The data type, must implement `bytemuck::Pod + Send + Sync`
    ///
    /// # Parameters
    /// - `data`: A vector containing the data to bind as a read-only storage buffer
    ///
    /// # Returns
    /// A new `InstancerBinding::StorageRO` containing the provided data.
    ///
    /// # Example
    /// ```rust
    /// let vertices = vec![Vertex::new(0.0, 0.0, 0.0); 1000];
    /// let binding = InstancerBinding::new_storage_ro(vertices);
    /// ```
    pub fn new_storage_ro<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::StorageRO(Box::new(data))
    }

    /// Creates a new read-write storage buffer binding from a vector of Pod data.
    ///
    /// Read-write storage buffers allow shaders to both read from and write to
    /// the buffer. This is useful for data that needs to be modified by compute
    /// shaders, such as particle positions or animation states.
    ///
    /// # Type Parameters
    /// - `T`: The data type, must implement `bytemuck::Pod + Send + Sync`
    ///
    /// # Parameters
    /// - `data`: A vector containing the data to bind as a read-write storage buffer
    ///
    /// # Returns
    /// A new `InstancerBinding::StorageRW` containing the provided data.
    ///
    /// # Example
    /// ```rust
    /// let particles = vec![Particle::default(); 10000];
    /// let binding = InstancerBinding::new_storage_rw(particles);
    /// ```
    pub fn new_storage_rw<T: bytemuck::Pod + Send + Sync + 'static>(data: Vec<T>) -> Self {
        Self::StorageRW(Box::new(data))
    }

    /// Returns a reference to the underlying data regardless of binding type.
    ///
    /// This method provides unified access to the data contained within any
    /// variant of the `InstancerBinding` enum.
    ///
    /// # Returns
    /// A reference to the `PodData` trait object containing the binding's data.
    fn data(&self) -> &dyn PodData {
        match self {
            InstancerBinding::Uniform(data) => data.as_ref(),
            InstancerBinding::StorageRO(data) => data.as_ref(),
            InstancerBinding::StorageRW(data) => data.as_ref(),
        }
    }

    /// Returns the appropriate GPU buffer binding type for this binding.
    ///
    /// Different binding variants require different GPU buffer binding types
    /// to properly configure the shader interface:
    /// - Uniform -> BufferBindingType::Uniform
    /// - StorageRO -> BufferBindingType::Storage { read_only: true }
    /// - StorageRW -> BufferBindingType::Storage { read_only: false }
    ///
    /// # Returns
    /// The `BufferBindingType` corresponding to this binding's access pattern.
    fn binding_type(&self) -> BufferBindingType {
        match self {
            InstancerBinding::Uniform(_) => BufferBindingType::Uniform,
            InstancerBinding::StorageRO(_) => BufferBindingType::Storage { read_only: true },
            InstancerBinding::StorageRW(_) => BufferBindingType::Storage { read_only: false },
        }
    }

    /// Returns the appropriate GPU buffer usage flags for this binding type.
    ///
    /// Different binding types require different usage flags to enable proper
    /// GPU operations:
    /// - Uniform buffers need UNIFORM | COPY_DST flags
    /// - Storage buffers need STORAGE | COPY_DST flags
    ///
    /// The COPY_DST flag is always included to allow CPU-to-GPU data transfers.
    ///
    /// # Returns
    /// The `BufferUsages` flags appropriate for this binding type.
    fn buffer_usages(&self) -> BufferUsages {
        match self {
            InstancerBinding::Uniform(_) => BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            _ => BufferUsages::STORAGE | BufferUsages::COPY_DST,
        }
    }
}

/// Function type for serial instance modification.
///
/// This function type defines the signature for CPU-based instance modification
/// functions used by the serial instancer. The function receives:
/// - A mutable slice of instances to modify
/// - Delta time (time since last frame) in seconds
/// - Total elapsed time in seconds
///
/// # Parameters
/// - `&mut [Instance]`: Mutable slice of instances to modify
/// - `f32`: Delta time in seconds
/// - `f32`: Total elapsed time in seconds
pub type SerialModifier = fn(&mut [Instance], f32, f32);

/// An instancer for managing and updating collections of instances.
///
/// **RECOMMENDATION: Use the serial instancer for production code.**
///
/// While this struct provides both serial and parallel instancing capabilities,
/// the parallel (GPU compute shader) instancer is currently in an unstable state
/// and suffers from significant performance issues. The parallel implementation
/// has not been fully optimized and may cause performance degradation compared
/// to the serial CPU-based approach.
///
/// The serial instancer, while running on the CPU, provides reliable and
/// predictable performance characteristics. It processes instances sequentially
/// using a provided modifier function and is the recommended approach until
/// the parallel instancer implementation is stabilized and optimized.
///
/// Use `Instancer::new_serial()` for stable, production-ready instancing,
/// and only consider `Instancer::new_parallel()` for experimental purposes
/// or if you specifically need GPU compute shader functionality and can
/// tolerate the current performance limitations.
pub struct Instancer {
    pub(crate) range: Option<Range<u64>>,
    pub(crate) instancer_kind: InstancerKind,
}

/// Internal enum representing different instancer implementation strategies.
///
/// This enum encapsulates the two different approaches to instance processing:
/// - Serial: CPU-based processing using a function pointer
/// - Parallel: GPU-based processing using compute shaders
pub(crate) enum InstancerKind {
    /// Serial processing variant that uses CPU-based instance modification.
    ///
    /// Contains a function pointer that will be called to modify instances
    /// on the CPU in a sequential manner.
    Serial { serial_modifier: SerialModifier },
    /// Parallel processing variant that uses GPU compute shaders.
    ///
    /// Contains all the GPU resources needed for compute shader execution:
    /// - Shader hash for caching and identification
    /// - Compiled shader module
    /// - Compute pipeline for execution
    /// - Bind group for resource binding (created lazily)
    /// - Buffers for shader data (time values, range, and user bindings)
    Parallel {
        shader_hash: String,
        pipeline: ComputePipeline,
        bind_group: Option<BindGroup>,
        buffers: Vec<Buffer>,
    },
}

impl Instancer {
    /// Creates a new serial (CPU-based) instancer.
    ///
    /// Serial instancers process instances sequentially on the CPU using a
    /// provided modifier function. This approach is currently more stable and
    /// performant than the parallel GPU-based approach and is recommended
    /// for production use.
    ///
    /// # Parameters
    /// - `range`: Optional range of instances to process. If `None`, all instances
    ///   in the model will be processed. The range is inclusive of start and
    ///   exclusive of end (standard Rust range semantics).
    /// - `serial_modifier`: Function that will be called to modify instances.
    ///   Receives a mutable slice of instances, delta time, and total time.
    ///
    /// # Returns
    /// A new `Instancer` configured for serial processing.
    ///
    /// # Example
    /// ```rust
    /// let instancer = Instancer::new_serial(
    ///     Some(0..100), // Process first 100 instances
    ///     |instances, delta_t, t| {
    ///         for instance in instances {
    ///             instance.pos(|pos| {
    ///                 pos.y += f32::sin(t) * delta_t;
    ///             });
    ///         }
    ///     }
    /// );
    /// ```
    pub fn new_serial(range: Option<Range<u64>>, serial_modifier: SerialModifier) -> Self {
        Self {
            range,
            instancer_kind: InstancerKind::Serial { serial_modifier },
        }
    }

    /// Creates a new parallel (GPU-based) instancer using compute shaders.
    ///
    /// **WARNING: This implementation is currently unstable and may have performance issues.**
    ///
    /// Parallel instancers use GPU compute shaders to process instances in parallel.
    /// While potentially more powerful for large datasets, the current implementation
    /// has stability and performance issues and should be used with caution.
    ///
    /// The instancer automatically creates several built-in uniform buffers that are
    /// available to your compute shader:
    /// - `@binding(0)`: Instance buffer (storage, read_write)
    /// - `@binding(1)`: Delta time (uniform, f32)
    /// - `@binding(2)`: Total time (uniform, f32)
    /// - `@binding(3)`: Range (uniform, vec2<i32>)
    /// - `@binding(4+)`: Your custom bindings in the order provided
    ///
    /// # Parameters
    /// - `range`: Optional range of instances to process. If `None`, the shader
    ///   should process all instances. Range values are passed to the shader
    ///   as a `vec2<i32>` where (-1, -1) indicates no range restriction.
    /// - `asset_server`: Reference to the asset server for GPU resource creation.
    /// - `bindings`: Vector of additional data bindings to make available to
    ///   the compute shader. These will be bound starting at binding 4.
    /// - `shader`: WGSL compute shader source code. Must have an entry point
    ///   named "main" with `@compute` attribute.
    ///
    /// # Returns
    /// A new `Instancer` configured for parallel GPU processing.
    ///
    /// # Panics
    /// Panics if GPU resource creation fails or if the shader compilation fails.
    ///
    /// # Example
    /// ```rust
    /// let shader_source = r#"
    ///     @group(0) @binding(0) var<storage, read_write> instances: array<InstanceData>;
    ///     @group(0) @binding(1) var<uniform> delta_t: f32;
    ///     @group(0) @binding(2) var<uniform> t: f32;
    ///
    ///     @compute @workgroup_size(64)
    ///     fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    ///         let index = id.x;
    ///         instances[index].position.y += sin(t) * delta_t;
    ///     }
    /// "#;
    ///
    /// let instancer = Instancer::new_parallel(
    ///     Some(0..1000),
    ///     &asset_server,
    ///     vec![], // No additional bindings
    ///     shader_source
    /// );
    /// ```
    pub fn new_parallel(
        range: Option<Range<u64>>,
        asset_server: &AssetServer,
        bindings: Vec<InstancerBinding>,
        shader: &str,
    ) -> Self {
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

        let range_i32 = if let Some(range) = range.as_ref() {
            (range.start as i32)..(range.end as i32)
        } else {
            -1..-1
        };

        buffers.push(
            asset_server
                .gpu_controller
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Instance Buffer Binding: range"),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    contents: bytemuck::cast_slice(&[range_i32.start, range_i32.end]),
                }),
        );

        // Create the buffers first
        for (binding_count, binding) in bindings.iter().enumerate() {
            buffers.push(
                asset_server
                    .gpu_controller
                    .create_buffer(&BufferDescriptor {
                        label: Some(&format!(
                            "Instancer Buffer Binding: {}",
                            binding_count + BUILTING_BUFFERS
                        )),
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
            // Range
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ]);

        bindings.iter().enumerate().for_each(|(index, binding)| {
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: index as u32 + 1,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: binding.binding_type(),
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            });
        });

        // Add the Bind group layout to the layouts manager
        asset_server
            .gpu_controller
            .write_layouts(|layouts| {
                layouts.insert(
                    shader_hash.clone(),
                    asset_server.gpu_controller.create_bind_group_layout(
                        &BindGroupLayoutDescriptor {
                            label: Some("Instancer"),
                            entries: &bind_group_layout_entries,
                        },
                    ),
                );
            })
            .unwrap_or_else(|err| {
                error!("Failed to write Instancer Layouts: {err}");
            });

        // Create the pipeline based on the new layout
        let pipeline = asset_server
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

                let pipeline_layout =
                    asset_server
                        .gpu_controller
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("Instancer Pipeline Layout"),
                            bind_group_layouts: &[bind_group_layout],
                            push_constant_ranges: &[],
                        });

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
            })
            .and_then(|res| {
                Ok(res.unwrap_or_else(|err| {
                    error!("Failed to create pipeline: {err}");
                    panic!();
                }))
            })
            .unwrap_or_else(|err| {
                error!("Failed to create pipeline: {err}");
                panic!();
            });

        Self {
            range,
            instancer_kind: InstancerKind::Parallel {
                shader_hash,
                pipeline,
                bind_group: None,
                buffers,
            },
        }
    }

    /// Prepares the instancer for execution by updating GPU resources and creating bind groups.
    ///
    /// This method is called internally before compute shader dispatch to ensure all GPU
    /// resources are properly configured and up-to-date. For parallel instancers, it:
    /// 1. Updates the delta time and total time uniform buffers with current values
    /// 2. Creates the bind group if it doesn't exist (lazy initialization)
    /// 3. Returns a reference to the bind group for compute pass binding
    ///
    /// For serial instancers, this method returns `None` as no GPU preparation is needed.
    ///
    /// # Parameters
    /// - `instance_buffer`: The GPU buffer containing instance data that will be modified
    /// - `gpu_controller`: Reference to the GPU controller for resource operations
    /// - `dt`: Delta time in seconds since the last frame
    /// - `t`: Total elapsed time in seconds since application start
    ///
    /// # Returns
    /// - `Some(&BindGroup)`: For parallel instancers, returns the bind group to use for compute dispatch
    /// - `None`: For serial instancers, as no GPU resources are needed
    ///
    /// # Notes
    /// The bind group is created lazily on first call to avoid unnecessary GPU resource
    /// allocation. Once created, it is reused for all subsequent calls.
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
