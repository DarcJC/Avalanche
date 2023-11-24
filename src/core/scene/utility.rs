
macro_rules! get_or_create_buf {
    ($func_name:ident, $get_cpu_func:ident, $buf:ident, $type:ty, $buffer_usage:expr) => {
        fn $func_name(&mut self) -> Arc<RefCell<T>> {
            let (buffer, created) = {
                let data = self.$get_cpu_func();
                let data_size = data.len() * std::mem::size_of::<$type>();
                let create_info = RendererType::get_buffer_creation_info(BitFlags::from(GraphicsBufferShareModes::Exclusive), BitFlags::from($buffer_usage), BitFlags::empty(), data_size).unwrap();
                get_or_create_buffer(&mut self.$buf, create_info).unwrap()
            };
            if created {
                let data = self.$get_cpu_func();
                let data_size = data.len() * std::mem::size_of::<$type>();
                let data_addr = data.as_ptr() as *const c_void;
                unsafe { buffer.borrow_mut().fill_buffer_on_device(data_addr, data_size).expect("Failed to copy data to buffer."); }
            }
            buffer
        }
    };
}

pub(crate) use get_or_create_buf;
