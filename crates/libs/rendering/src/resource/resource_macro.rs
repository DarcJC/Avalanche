#[cfg(debug_assertions)]
#[macro_export]
macro_rules! render_resource_wrapper {
    ($wrapper_type:ident, $wgpu_type:ty) => {
        #[derive(Debug)]
        // SAFETY: while self is live, self.0 comes from `into_raw` of an Arc<$wgpu_type> with a strong ref.
        pub struct $wrapper_type(*const ());

        impl $wrapper_type {
            pub fn new(value: $wgpu_type) -> Self {
                let arc = std::sync::Arc::new(value);
                let value_ptr = std::sync::Arc::into_raw(arc);
                let unit_ptr = value_ptr.cast::<()>();
                Self(unit_ptr)
            }

            pub fn try_unwrap(self) -> Option<$wgpu_type> {
                let value_ptr = self.0.cast::<$wgpu_type>();
                // SAFETY: pointer refers to a valid Arc, and was created from Arc::into_raw.
                let arc = unsafe { std::sync::Arc::from_raw(value_ptr) };

                // we forget ourselves here since the reconstructed arc will be dropped/decremented within this scope
                std::mem::forget(self);

                std::sync::Arc::try_unwrap(arc).ok()
            }
        }

        impl std::ops::Deref for $wrapper_type {
            type Target = $wgpu_type;

            fn deref(&self) -> &Self::Target {
                let value_ptr = self.0.cast::<$wgpu_type>();
                // SAFETY: the arc lives for 'self, so the ref lives for 'self
                let value_ref = unsafe { value_ptr.as_ref() };
                value_ref.unwrap()
            }
        }

        impl Drop for $wrapper_type {
            fn drop(&mut self) {
                let value_ptr = self.0.cast::<$wgpu_type>();
                // SAFETY: pointer refers to a valid Arc, and was created from Arc::into_raw.
                // this reconstructed arc is dropped/decremented within this scope.
                unsafe { std::sync::Arc::from_raw(value_ptr) };
            }
        }

        // SAFETY: We manually implement Send and Sync, which is valid for Arc<T> when T: Send + Sync.
        // We ensure correctness by checking that $wgpu_type does implement Send and Sync.
        // If in future there is a case where a wrapper is required for a non-send/sync type
        // we can implement a macro variant that omits these manual Send + Sync impls
        unsafe impl Send for $wrapper_type {}
        // SAFETY: As explained above, we ensure correctness by checking that $wgpu_type implements Send and Sync.
        unsafe impl Sync for $wrapper_type {}
        const _: () = {
            trait AssertSendSyncBound: Send + Sync {}
            impl AssertSendSyncBound for $wgpu_type {}
        };

        impl Clone for $wrapper_type {
            fn clone(&self) -> Self {
                let value_ptr = self.0.cast::<$wgpu_type>();
                // SAFETY: pointer refers to a valid Arc, and was created from Arc::into_raw.
                let arc = unsafe { std::sync::Arc::from_raw(value_ptr.cast::<$wgpu_type>()) };
                let cloned = std::sync::Arc::clone(&arc);
                // we forget the reconstructed Arc to avoid decrementing the ref counter, as self is still live.
                std::mem::forget(arc);
                let cloned_value_ptr = std::sync::Arc::into_raw(cloned);
                let cloned_unit_ptr = cloned_value_ptr.cast::<()>();
                Self(cloned_unit_ptr)
            }
        }
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! render_resource_wrapper {
    ($wrapper_type:ident, $wgpu_type:ty) => {
        #[derive(Clone, Debug)]
        pub struct $wrapper_type(std::sync::Arc<$wgpu_type>);

        impl $wrapper_type {
            pub fn new(value: $wgpu_type) -> Self {
                Self(std::sync::Arc::new(value))
            }

            pub fn try_unwrap(self) -> Option<$wgpu_type> {
                std::sync::Arc::try_unwrap(self.0).ok()
            }
        }

        impl std::ops::Deref for $wrapper_type {
            type Target = $wgpu_type;

            fn deref(&self) -> &Self::Target {
                self.0.as_ref()
            }
        }

        const _: () = {
            trait AssertSendSyncBound: Send + Sync {}
            impl AssertSendSyncBound for $wgpu_type {}
        };
    };
}
