#[cfg(feature = "parking_lot")]
type InnerLock<T> = parking_lot::RwLock<T>;

#[cfg(not(feature = "parking_lot"))]
type InnerLock<T> = std::sync::RwLock<T>;

#[derive(Debug)]
pub struct RwLock<T>(InnerLock<T>);

impl<T> RwLock<T> {
    pub fn new(value: T) -> Self {
        Self(InnerLock::new(value))
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        #[cfg(feature = "parking_lot")]
        {
            RwLockReadGuard(self.0.read())
        }

        #[cfg(not(feature = "parking_lot"))]
        {
            RwLockReadGuard(self.0.read().unwrap())
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        #[cfg(feature = "parking_lot")]
        {
            RwLockWriteGuard(self.0.write())
        }

        #[cfg(not(feature = "parking_lot"))]
        {
            RwLockWriteGuard(self.0.write().unwrap())
        }
    }
}

macro_rules! guard_wrapper {
    (read, $name:ident, $inner:ty) => {
        pub struct $name<'a, T: 'a>($inner);

        impl<'a, T> std::ops::Deref for $name<'a, T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }
    };
    (write, $name:ident, $inner:ty) => {
        pub struct $name<'a, T: 'a>($inner);

        impl<'a, T> std::ops::Deref for $name<'a, T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        impl<'a, T> std::ops::DerefMut for $name<'a, T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut *self.0
            }
        }
    };
}

#[cfg(feature = "parking_lot")]
guard_wrapper!(read, RwLockReadGuard, parking_lot::RwLockReadGuard<'a, T>);
#[cfg(not(feature = "parking_lot"))]
guard_wrapper!(read, RwLockReadGuard, std::sync::RwLockReadGuard<'a, T>);

#[cfg(feature = "parking_lot")]
guard_wrapper!(
    write,
    RwLockWriteGuard,
    parking_lot::RwLockWriteGuard<'a, T>
);
#[cfg(not(feature = "parking_lot"))]
guard_wrapper!(write, RwLockWriteGuard, std::sync::RwLockWriteGuard<'a, T>);
