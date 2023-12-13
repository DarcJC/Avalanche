#[macro_export]
macro_rules! static_accessors {
    ($lifetime:lifetime $vis:vis $static_name:ident, $fn_name:ident, $fn_name_mut:ident, $fn_name_sync:ident, $fn_name_mut_sync:ident, $type:ty) => {

        $vis async fn $fn_name() -> async_std::sync::RwLockReadGuard<$lifetime, $type> {
            $static_name.read().await
        }

        $vis async fn $fn_name_mut() -> async_std::sync::RwLockWriteGuard<$lifetime, $type> {
            $static_name.write().await
        }

        $vis fn $fn_name_sync() -> async_std::sync::RwLockReadGuard<$lifetime, $type> {
            async_std::task::block_on($static_name.read())
        }

        $vis fn $fn_name_mut_sync() -> async_std::sync::RwLockWriteGuard<$lifetime, $type> {
            async_std::task::block_on($static_name.write())
        }
    };
}
