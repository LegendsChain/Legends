//! # Template Pallet
//! 
//! 这是一个用于 BitNice 区块链的模板 pallet。
//! 
//! ## 概述
//! 
//! 这个 pallet 提供了基础的功能示例，可以作为开发新 pallet 的起点。

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 配置接口 - 定义此 pallet 与运行时集成所需的类型和常量
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 由于此 pallet 会发出事件，因此依赖于运行时的事件类型定义
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 执行操作的权重信息
        type WeightInfo: WeightInfo;
    }

    /// Pallet 存储项 - 用于存储单个值
    #[pallet::storage]
    #[pallet::getter(fn something)]
    pub type Something<T> = StorageValue<_, u32>;

    /// Pallet 事件
    /// 当特定操作发生时发出的事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 成功存储了一个值
        SomethingStored { 
            /// 存储的值
            something: u32, 
            /// 执行操作的账户
            who: T::AccountId 
        },
    }

    /// Pallet 的错误类型
    #[pallet::error]
    pub enum Error<T> {
        /// 没有找到值
        NoneValue,
        /// 值已经存在，无法设置
        StorageOverflow,
    }

    /// 可调用函数接口
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 示例可调用函数 - 存储给定的数字
        /// 
        /// 此函数允许用户存储一个数字到链状态中。
        /// 
        /// 参数:
        /// - `origin`: 交易的发起者
        /// - `something`: 要存储的数字
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            // 检查交易是否由签名用户发起
            let who = ensure_signed(origin)?;

            // 更新存储
            <Something<T>>::put(&something);

            // 发出事件
            Self::deposit_event(Event::SomethingStored { something, who });

            // 返回成功
            Ok(())
        }

        /// 另一个示例函数 - 引发错误
        /// 
        /// 此函数总是失败，用于演示错误处理。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // 读取一个值，如果没有则返回错误
            match <Something<T>>::get() {
                None => return Err(Error::<T>::NoneValue.into()),
                Some(old) => {
                    // 检查溢出
                    let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
                    <Something<T>>::put(&new);
                }
            }
            Ok(())
        }
    }
}

/// 权重信息的抽象接口
pub trait WeightInfo {
    fn do_something() -> Weight;
    fn cause_error() -> Weight;
}

/// 默认权重实现
impl WeightInfo for () {
    fn do_something() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn cause_error() -> Weight {
        Weight::from_parts(10_000, 0)
    }
} 