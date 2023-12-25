use anyhow::Result;
use ash::vk;
use std::{
    ffi::{c_char, c_void},
    mem::align_of,
};

pub struct Buffer {
    mapped: Option<*mut c_void>,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    buffer_size: vk::DeviceSize,
    instance_count: usize,
    instance_size: vk::DeviceSize,
    alignment_size: vk::DeviceSize,
    usage_flags: vk::BufferUsageFlags,
    memory_property_flags: vk::MemoryPropertyFlags,
}

impl Buffer {
    pub fn new(
        device: &crate::Device,
        instance_size: vk::DeviceSize,
        instance_count: usize,
        usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        min_offset_alignment: Option<vk::DeviceSize>,
    ) -> Result<Self> {
        let min_offset_alignment = match min_offset_alignment {
            Some(offset) => offset,
            None => 0,
        };
        let alignment_size = Self::alignment(instance_size, min_offset_alignment);
        let buffer_size = alignment_size * instance_count as u64;
        let (buffer, memory) =
            device.create_buffer(buffer_size, usage_flags, memory_property_flags)?;

        Ok(Self {
            mapped: None,
            buffer,
            memory,
            buffer_size,
            instance_size,
            instance_count,
            alignment_size,
            usage_flags,
            memory_property_flags,
        })
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        let device_ref = device.device();

        self.unmap(device);
        device_ref.destroy_buffer(self.buffer, None);
        device_ref.free_memory(self.memory, None);
    }

    pub unsafe fn map(
        &mut self,
        device: &crate::Device,
        size: Option<vk::DeviceSize>,
        offset: Option<vk::DeviceSize>,
    ) -> Result<()> {
        assert!(
            (self.buffer != vk::Buffer::null()) && (self.memory != vk::DeviceMemory::null()),
            "Called map on buffer before create"
        );

        let device_ref = device.device();
        let size = match size {
            Some(size) => size,
            None => vk::WHOLE_SIZE,
        };
        let offset = match offset {
            Some(offset) => offset,
            None => 0,
        };

        self.mapped =
            Some(device_ref.map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())?);

        Ok(())
    }

    pub unsafe fn unmap(&mut self, device: &crate::Device) {
        if self.mapped.is_some() {
            let device_ref = device.device();

            device_ref.unmap_memory(self.memory);
            self.mapped = None;
        }
    }

    pub unsafe fn write_to_buffer<T: Copy + Clone>(
        &mut self,
        data: *const c_void,
        size: Option<vk::DeviceSize>,
        offset: Option<vk::DeviceSize>,
    ) {
        assert!(self.mapped.is_some(), "Cannot copy to unmapped buffer");

        let size = match size {
            Some(size) => size,
            None => vk::WHOLE_SIZE,
        };
        let offset = match offset {
            Some(offset) => offset,
            None => 0,
        };
        // The asssertion above promises that the buffer is mapped
        let mapped = self.mapped.unwrap();

        if size == vk::WHOLE_SIZE {
            let mut align =
                ash::util::Align::<T>::new(mapped, align_of::<T>() as u64, self.buffer_size);

            align.copy_from_slice(std::slice::from_raw_parts(
                data as *const T,
                self.buffer_size as usize,
            ));
        } else {
            let mem_offset = (mapped as *mut c_char).wrapping_add(offset as usize);
            let mut align =
                ash::util::Align::<T>::new(mem_offset as *mut c_void, align_of::<T>() as u64, size);

            align.copy_from_slice(std::slice::from_raw_parts(data as *const T, size as usize));
        }
    }

    pub unsafe fn flush(
        &self,
        device: &crate::Device,
        size: Option<vk::DeviceSize>,
        offset: Option<vk::DeviceSize>,
    ) -> Result<()> {
        let device_ref = device.device();
        let mapped_range = vk::MappedMemoryRange::builder()
            .memory(self.memory)
            .offset(offset.unwrap_or(0))
            .size(size.unwrap_or(vk::WHOLE_SIZE));

        device_ref.flush_mapped_memory_ranges(std::slice::from_ref(&mapped_range))?;

        Ok(())
    }

    pub fn descriptor_info(
        &self,
        size: Option<vk::DeviceSize>,
        offset: Option<vk::DeviceSize>,
    ) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset(offset.unwrap_or(0))
            .range(size.unwrap_or(vk::WHOLE_SIZE))
            .build()
    }

    pub unsafe fn invalidate(
        &self,
        device: &crate::Device,
        size: Option<vk::DeviceSize>,
        offset: Option<vk::DeviceSize>,
    ) -> Result<()> {
        let device_ref = device.device();
        let mapped_range = vk::MappedMemoryRange::builder()
            .memory(self.memory)
            .offset(offset.unwrap_or(0))
            .size(size.unwrap_or(vk::WHOLE_SIZE));

        device_ref.invalidate_mapped_memory_ranges(std::slice::from_ref(&mapped_range))?;

        Ok(())
    }

    pub unsafe fn write_to_index<T: Clone + Copy>(
        &mut self,
        data: *const c_void,
        index: vk::DeviceSize,
    ) {
        self.write_to_buffer::<T>(
            data,
            Some(self.instance_size),
            Some(index * self.alignment_size),
        )
    }

    pub unsafe fn flush_index(&self, device: &crate::Device, index: vk::DeviceSize) -> Result<()> {
        self.flush(
            device,
            Some(self.alignment_size),
            Some(index * self.alignment_size),
        )
    }

    pub fn descriptor_info_for_index(&self, index: vk::DeviceSize) -> vk::DescriptorBufferInfo {
        self.descriptor_info(Some(self.alignment_size), Some(index * self.alignment_size))
    }

    pub unsafe fn invalidate_index(
        &self,
        device: &crate::Device,
        index: vk::DeviceSize,
    ) -> Result<()> {
        self.invalidate(
            device,
            Some(self.alignment_size),
            Some(index * self.alignment_size),
        )
    }

    pub fn buffer(&self) -> &vk::Buffer {
        &self.buffer
    }

    pub fn mapped_memory(&self) -> Option<*mut c_void> {
        self.mapped
    }

    pub fn instance_count(&self) -> u32 {
        self.instance_count as u32
    }

    pub fn instance_size(&self) -> vk::DeviceSize {
        self.instance_size
    }

    pub fn alignment_size(&self) -> vk::DeviceSize {
        self.alignment_size
    }

    pub fn usage_flags(&self) -> vk::BufferUsageFlags {
        self.usage_flags
    }

    pub fn memory_property_flags(&self) -> vk::MemoryPropertyFlags {
        self.memory_property_flags
    }

    pub fn buffer_size(&self) -> vk::DeviceSize {
        self.buffer_size
    }

    fn alignment(
        instance_size: vk::DeviceSize,
        min_offset_alignment: vk::DeviceSize,
    ) -> vk::DeviceSize {
        if min_offset_alignment > 0 {
            (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1)
        } else {
            instance_size
        }
    }
}
