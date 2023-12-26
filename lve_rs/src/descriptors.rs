use anyhow::Result;
use ash::vk;
use std::collections::HashMap;

pub struct DescriptorSetLayoutBuilder {
    bindings: HashMap<u32, vk::DescriptorSetLayoutBinding>,
}

pub struct DescriptorSetLayout {
    descriptor_set_layout: vk::DescriptorSetLayout,
    bindings: HashMap<u32, vk::DescriptorSetLayoutBinding>,
}

pub struct DescriptorPool {
    descriptor_pool: vk::DescriptorPool,
}

pub struct DescriptorPoolBuilder {
    pool_sizes: Vec<vk::DescriptorPoolSize>,
    max_sets: u32,
    pool_flags: vk::DescriptorPoolCreateFlags,
}

pub struct DescriptorWriter<'a> {
    set_layout: &'a DescriptorSetLayout,
    pool: &'a DescriptorPool,
    writes: Vec<vk::WriteDescriptorSet>,
}

impl DescriptorSetLayoutBuilder {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn add_binding(
        &mut self,
        binding: u32,
        descriptor_type: vk::DescriptorType,
        stage_flags: vk::ShaderStageFlags,
        count: Option<u32>,
    ) -> Self {
        assert!(
            !self.bindings.contains_key(&binding),
            "Binding already in use"
        );

        let count = count.unwrap_or(1);
        let layout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(count)
            .stage_flags(stage_flags)
            .build();

        self.bindings.insert(binding, layout_binding);

        Self {
            bindings: self.bindings.clone(),
        }
    }

    pub fn build(&self, device: &crate::Device) -> Result<Box<DescriptorSetLayout>> {
        Ok(Box::new(DescriptorSetLayout::new(device, &self.bindings)?))
    }
}

impl DescriptorSetLayout {
    pub fn new(
        device: &crate::Device,
        bindings: &HashMap<u32, vk::DescriptorSetLayoutBinding>,
    ) -> Result<Self> {
        let device = device.device();
        let set_layout_bindings = bindings.iter().map(|kv| *kv.1).collect::<Vec<_>>();
        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&set_layout_bindings);
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&descriptor_set_layout_info, None) }?;

        Ok(Self {
            descriptor_set_layout,
            bindings: bindings.clone(),
        })
    }

    pub fn builder() -> DescriptorSetLayoutBuilder {
        DescriptorSetLayoutBuilder::new()
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device = device.device();

        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl DescriptorPoolBuilder {
    pub fn new() -> Self {
        Self {
            pool_sizes: vec![],
            max_sets: 1000,
            pool_flags: vk::DescriptorPoolCreateFlags::empty(),
        }
    }

    pub fn add_pool_size(&self, descriptor_type: vk::DescriptorType, count: u32) -> Self {
        let mut pool_sizes = self.pool_sizes.clone();

        pool_sizes.push(vk::DescriptorPoolSize {
            ty: descriptor_type,
            descriptor_count: count,
        });

        Self {
            pool_sizes,
            max_sets: self.max_sets,
            pool_flags: self.pool_flags,
        }
    }

    pub fn set_pool_flags(&self, flags: vk::DescriptorPoolCreateFlags) -> Self {
        Self {
            pool_flags: flags,
            pool_sizes: self.pool_sizes.clone(),
            max_sets: self.max_sets,
        }
    }

    pub fn set_max_sets(&self, count: u32) -> Self {
        Self {
            max_sets: count,
            pool_sizes: self.pool_sizes.clone(),
            pool_flags: self.pool_flags,
        }
    }

    pub fn build(&self, device: &crate::Device) -> Result<Box<DescriptorPool>> {
        Ok(Box::new(DescriptorPool::new(
            device,
            self.max_sets,
            self.pool_flags,
            &self.pool_sizes,
        )?))
    }
}

impl DescriptorPool {
    pub fn new(
        device: &crate::Device,
        max_sets: u32,
        pool_flags: vk::DescriptorPoolCreateFlags,
        pool_sizes: &Vec<vk::DescriptorPoolSize>,
    ) -> Result<Self> {
        let device = device.device();
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(max_sets)
            .flags(pool_flags);
        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) }?;

        Ok(Self { descriptor_pool })
    }

    pub fn builder() -> DescriptorPoolBuilder {
        DescriptorPoolBuilder::new()
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device = device.device();

        device.destroy_descriptor_pool(self.descriptor_pool, None);
    }

    pub unsafe fn allocate_descriptor(
        &self,
        device: &crate::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> (Vec<vk::DescriptorSet>, bool) {
        let device = device.device();
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        match device.allocate_descriptor_sets(&alloc_info) {
            Ok(sets) => (sets, true),
            Err(_) => (vec![], false),
        }
    }

    pub unsafe fn free_descriptors(
        &self,
        device: &crate::Device,
        descriptors: &Vec<vk::DescriptorSet>,
    ) -> Result<()> {
        let device = device.device();

        device.free_descriptor_sets(self.descriptor_pool, descriptors)?;

        Ok(())
    }

    pub unsafe fn reset_pool(&self, device: &crate::Device) -> Result<()> {
        let device = device.device();

        device
            .reset_descriptor_pool(self.descriptor_pool, vk::DescriptorPoolResetFlags::empty())?;

        Ok(())
    }
}

impl<'a> DescriptorWriter<'a> {
    pub fn new(set_layout: &'a DescriptorSetLayout, pool: &'a DescriptorPool) -> Self {
        Self {
            set_layout,
            pool,
            writes: vec![],
        }
    }

    pub fn write_buffer(&self, binding: u32, buffer_info: &vk::DescriptorBufferInfo) -> Self {
        let mut writes = self.writes.clone();
        assert!(
            self.set_layout.bindings.contains_key(&binding),
            "Layout does not contain specified binding"
        );

        let binding_description = &self.set_layout.bindings[&binding];

        assert!(
            binding_description.descriptor_count == 1,
            "Binding single descriptor info, but binding expects multiple"
        );

        let write = vk::WriteDescriptorSet::builder()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .buffer_info(std::slice::from_ref(buffer_info))
            .build();

        writes.push(write);

        Self {
            set_layout: self.set_layout,
            pool: self.pool,
            writes,
        }
    }

    pub fn write_image(&self, binding: u32, image_info: &vk::DescriptorImageInfo) -> Self {
        let mut writes = self.writes.clone();

        assert!(
            self.set_layout.bindings.contains_key(&binding),
            "Layout does not contain specified binding"
        );

        let binding_description = &self.set_layout.bindings[&binding];

        assert!(
            binding_description.descriptor_count == 1,
            "Binding single descriptor info, but binding expects multiple"
        );

        let write = vk::WriteDescriptorSet::builder()
            .descriptor_type(binding_description.descriptor_type)
            .dst_binding(binding)
            .image_info(std::slice::from_ref(image_info))
            .build();

        writes.push(write);

        Self {
            set_layout: self.set_layout,
            pool: self.pool,
            writes,
        }
    }

    pub unsafe fn build(&mut self, device: &crate::Device) -> (vk::DescriptorSet, bool) {
        let (sets, success) = self
            .pool
            .allocate_descriptor(device, self.set_layout.descriptor_set_layout);

        if let Some(set) = sets.iter().next() {
            if success {
                self.overwrite(device, set);
            }

            return (*set, success);
        }

        (vk::DescriptorSet::null(), success)
    }

    pub unsafe fn overwrite(&mut self, device: &crate::Device, set: &vk::DescriptorSet) {
        let device = device.device();

        self.writes
            .iter_mut()
            .for_each(|write| write.dst_set = *set);

        device.update_descriptor_sets(&self.writes, &[])
    }
}
