pub struct Profiler {
    pub timestamps: Vec<QueryTimestampPass>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self { timestamps: vec![] }
    }
}

impl Profiler {
    pub fn add_query_timestamp_pass(&mut self, timestamp: QueryTimestampPass) {
        self.timestamps.push(timestamp);
    }
}

pub struct QueryTimestampPass {
    name: Option<&'static str>,
    pub pass_time: f64,
    pub query_timing: wgpu::QuerySet,
    query_resolve_buffer: wgpu::Buffer,
    query_result_buffer: wgpu::Buffer,
}

impl QueryTimestampPass {
    pub fn new(name: Option<&'static str>, device: &wgpu::Device) -> Self {
        let query_timing = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some(&format!("Query set {} timing", name.unwrap())),
            count: 2,
            ty: wgpu::QueryType::Timestamp,
        });

        let query_resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Query set buffer {} timing", name.unwrap())),
            size: 2 * 8,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Query resolve buffer {} result", name.unwrap())),
            size: query_resolve_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            name,
            pass_time: 0.0,
            query_timing,
            query_resolve_buffer,
            query_result_buffer,
        }
    }

    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(&self.query_timing, 0..2, &self.query_resolve_buffer, 0);

        encoder.copy_buffer_to_buffer(
            &self.query_resolve_buffer,
            0,
            &self.query_result_buffer,
            0,
            self.query_result_buffer.size(),
        );
    }

    pub fn map(&self) {
        let _ = self
            .query_result_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| {});
    }

    pub fn unmap(&mut self) {
        {
            let slice: &[u8] = &self.query_result_buffer.slice(..).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&slice);

            self.pass_time = (timestamps[1].wrapping_sub(timestamps[0]) / 1000) as f64;
        }
        self.query_result_buffer.unmap();
    }
}
