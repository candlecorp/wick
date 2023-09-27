#[allow(unsafe_code, unused, unused_qualifications, unreachable_pub, clippy::manual_assert)]

pub(crate) mod generated {
  #![allow(clippy::future_not_send, clippy::impl_trait_in_params)]
  use wasmtime::component::bindgen;
  pub struct Component {}
  const _: () = {
    use wasmtime::component::__internal::anyhow;
    impl Component {
      pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::component::Linker<T>,
        get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
      ) -> wasmtime::Result<()>
      where
        U: candle::wick::wick::Host + Send,
        T: Send,
      {
        candle::wick::wick::add_to_linker(linker, get)?;
        Ok(())
      }
      /// Instantiates the provided `module` using the specified
      /// parameters, wrapping up the result in a structure that
      /// translates between wasm and the host.
      pub async fn instantiate_async<T: Send>(
        mut store: impl wasmtime::AsContextMut<Data = T>,
        component: &wasmtime::component::Component,
        linker: &wasmtime::component::Linker<T>,
      ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
        let instance = linker.instantiate_async(&mut store, component).await?;
        Ok((Self::new(store, &instance)?, instance))
      }
      /// Instantiates a pre-instantiated module using the specified
      /// parameters, wrapping up the result in a structure that
      /// translates between wasm and the host.
      pub async fn instantiate_pre<T: Send>(
        mut store: impl wasmtime::AsContextMut<Data = T>,
        instance_pre: &wasmtime::component::InstancePre<T>,
      ) -> wasmtime::Result<(Self, wasmtime::component::Instance)> {
        let instance = instance_pre.instantiate_async(&mut store).await?;
        Ok((Self::new(store, &instance)?, instance))
      }
      /// Low-level creation wrapper for wrapping up the exports
      /// of the `instance` provided in this structure of wasm
      /// exports.
      ///
      /// This function will extract exports from the `instance`
      /// defined within `store` and wrap them all up in the
      /// returned structure which can be used to interact with
      /// the wasm module.
      pub fn new(
        mut store: impl wasmtime::AsContextMut,
        instance: &wasmtime::component::Instance,
      ) -> wasmtime::Result<Self> {
        let mut store = store.as_context_mut();
        let mut exports = instance.exports(&mut store);
        let mut __exports = exports.root();
        Ok(Component {})
      }
    }
  };
  pub mod candle {
    pub mod wick {
      #[allow(clippy::all)]
      pub mod wick {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::anyhow;
        pub type Json = String;
        const _: () = {
          if !(8 == <Json as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 8 == <Json as wasmtime::component::ComponentType>::SIZE32")
          }
          if !(4 == <Json as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 4 == <Json as wasmtime::component::ComponentType>::ALIGN32")
          }
        };
        pub struct Operation {
          pub name: String,
          pub component: String,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Operation {
          #[inline]
          fn clone(&self) -> Operation {
            Operation {
              name: ::core::clone::Clone::clone(&self.name),
              component: ::core::clone::Clone::clone(&self.component),
            }
          }
        }
        unsafe impl wasmtime::component::Lower for Operation {
          #[inline]
          fn lower<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            dst: &mut std::mem::MaybeUninit<Self::Lower>,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::lower(&self.name, cx, ty.fields[0usize].ty, {
              #[allow(unused_unsafe)]
              {
                unsafe {
                  use ::wasmtime::component::__internal::MaybeUninitExt;
                  let m: &mut std::mem::MaybeUninit<_> = dst;
                  m.map(|p| &mut (*p).name)
                }
              }
            })?;
            wasmtime::component::Lower::lower(&self.component, cx, ty.fields[1usize].ty, {
              #[allow(unused_unsafe)]
              {
                unsafe {
                  use ::wasmtime::component::__internal::MaybeUninitExt;
                  let m: &mut std::mem::MaybeUninit<_> = dst;
                  m.map(|p| &mut (*p).component)
                }
              }
            })?;
            Ok(())
          }
          #[inline]
          fn store<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            mut offset: usize,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            if true {
              if !(offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                  "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                )
              }
            }
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::store(
              &self.name,
              cx,
              ty.fields[0usize].ty,
              <String as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset),
            )?;
            wasmtime::component::Lower::store(
              &self.component,
              cx,
              ty.fields[1usize].ty,
              <String as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset),
            )?;
            Ok(())
          }
        }
        unsafe impl wasmtime::component::Lift for Operation {
          #[inline]
          fn lift(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            src: &Self::Lower,
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            Ok(Self {
              name: <String as wasmtime::component::Lift>::lift(cx, ty.fields[0usize].ty, &src.name)?,
              component: <String as wasmtime::component::Lift>::lift(cx, ty.fields[1usize].ty, &src.component)?,
            })
          }
          #[inline]
          fn load(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            bytes: &[u8],
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            if true {
              if !((bytes.as_ptr() as usize) % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                                      "assertion failed: (bytes.as_ptr() as usize) %\\n        (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                                  )
              }
            }
            let mut offset = 0;
            Ok(Self {
              name: <String as wasmtime::component::Lift>::load(
                cx,
                ty.fields[0usize].ty,
                &bytes[<String as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset)..]
                  [..<String as wasmtime::component::ComponentType>::SIZE32],
              )?,
              component: <String as wasmtime::component::Lift>::load(
                cx,
                ty.fields[1usize].ty,
                &bytes[<String as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset)..]
                  [..<String as wasmtime::component::ComponentType>::SIZE32],
              )?,
            })
          }
        }
        const _: () = {
          #[doc(hidden)]
          #[repr(C)]
          pub struct LowerOperation<T0: Copy, T1: Copy> {
            name: T0,
            component: T1,
            _align: [wasmtime::ValRaw; 0],
          }
          #[automatically_derived]
          impl<T0: ::core::clone::Clone + Copy, T1: ::core::clone::Clone + Copy> ::core::clone::Clone for LowerOperation<T0, T1> {
            #[inline]
            fn clone(&self) -> LowerOperation<T0, T1> {
              LowerOperation {
                name: ::core::clone::Clone::clone(&self.name),
                component: ::core::clone::Clone::clone(&self.component),
                _align: ::core::clone::Clone::clone(&self._align),
              }
            }
          }
          #[automatically_derived]
          impl<T0: ::core::marker::Copy + Copy, T1: ::core::marker::Copy + Copy> ::core::marker::Copy for LowerOperation<T0, T1> {}
          unsafe impl wasmtime::component::ComponentType for Operation {
            type Lower = LowerOperation<
              <String as wasmtime::component::ComponentType>::Lower,
              <String as wasmtime::component::ComponentType>::Lower,
            >;
            const ABI: wasmtime::component::__internal::CanonicalAbiInfo =
              wasmtime::component::__internal::CanonicalAbiInfo::record_static(&[
                <String as wasmtime::component::ComponentType>::ABI,
                <String as wasmtime::component::ComponentType>::ABI,
              ]);
            #[inline]
            fn typecheck(
              ty: &wasmtime::component::__internal::InterfaceType,
              types: &wasmtime::component::__internal::InstanceType<'_>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
              wasmtime::component::__internal::typecheck_record(
                ty,
                types,
                &[
                  ("name", <String as wasmtime::component::ComponentType>::typecheck),
                  ("component", <String as wasmtime::component::ComponentType>::typecheck),
                ],
              )
            }
          }
        };
        impl core::fmt::Debug for Operation {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Operation")
              .field("name", &self.name)
              .field("component", &self.component)
              .finish()
          }
        }
        const _: () = {
          if !(16 == <Operation as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 16 == <Operation as wasmtime::component::ComponentType>::SIZE32",)
          }
          if !(4 == <Operation as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 4 == <Operation as wasmtime::component::ComponentType>::ALIGN32",)
          }
        };
        pub struct Invocation {
          pub target: Operation,
          pub payload: Json,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Invocation {
          #[inline]
          fn clone(&self) -> Invocation {
            Invocation {
              target: ::core::clone::Clone::clone(&self.target),
              payload: ::core::clone::Clone::clone(&self.payload),
            }
          }
        }
        unsafe impl wasmtime::component::Lower for Invocation {
          #[inline]
          fn lower<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            dst: &mut std::mem::MaybeUninit<Self::Lower>,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::lower(&self.target, cx, ty.fields[0usize].ty, {
              #[allow(unused_unsafe)]
              {
                unsafe {
                  use ::wasmtime::component::__internal::MaybeUninitExt;
                  let m: &mut std::mem::MaybeUninit<_> = dst;
                  m.map(|p| &mut (*p).target)
                }
              }
            })?;
            wasmtime::component::Lower::lower(&self.payload, cx, ty.fields[1usize].ty, {
              #[allow(unused_unsafe)]
              {
                unsafe {
                  use ::wasmtime::component::__internal::MaybeUninitExt;
                  let m: &mut std::mem::MaybeUninit<_> = dst;
                  m.map(|p| &mut (*p).payload)
                }
              }
            })?;
            Ok(())
          }
          #[inline]
          fn store<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            mut offset: usize,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            if true {
              if !(offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                  "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                )
              }
            }
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::store(
              &self.target,
              cx,
              ty.fields[0usize].ty,
              <Operation as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset),
            )?;
            wasmtime::component::Lower::store(
              &self.payload,
              cx,
              ty.fields[1usize].ty,
              <Json as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset),
            )?;
            Ok(())
          }
        }
        unsafe impl wasmtime::component::Lift for Invocation {
          #[inline]
          fn lift(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            src: &Self::Lower,
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            Ok(Self {
              target: <Operation as wasmtime::component::Lift>::lift(cx, ty.fields[0usize].ty, &src.target)?,
              payload: <Json as wasmtime::component::Lift>::lift(cx, ty.fields[1usize].ty, &src.payload)?,
            })
          }
          #[inline]
          fn load(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            bytes: &[u8],
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            if true {
              if !((bytes.as_ptr() as usize) % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                                      "assertion failed: (bytes.as_ptr() as usize) %\\n        (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                                  )
              }
            }
            let mut offset = 0;
            Ok(Self {
              target: <Operation as wasmtime::component::Lift>::load(
                cx,
                ty.fields[0usize].ty,
                &bytes[<Operation as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset)..]
                  [..<Operation as wasmtime::component::ComponentType>::SIZE32],
              )?,
              payload: <Json as wasmtime::component::Lift>::load(
                cx,
                ty.fields[1usize].ty,
                &bytes[<Json as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset)..]
                  [..<Json as wasmtime::component::ComponentType>::SIZE32],
              )?,
            })
          }
        }
        const _: () = {
          #[doc(hidden)]
          #[repr(C)]
          pub struct LowerInvocation<T0: Copy, T1: Copy> {
            target: T0,
            payload: T1,
            _align: [wasmtime::ValRaw; 0],
          }
          #[automatically_derived]
          impl<T0: ::core::clone::Clone + Copy, T1: ::core::clone::Clone + Copy> ::core::clone::Clone
            for LowerInvocation<T0, T1>
          {
            #[inline]
            fn clone(&self) -> LowerInvocation<T0, T1> {
              LowerInvocation {
                target: ::core::clone::Clone::clone(&self.target),
                payload: ::core::clone::Clone::clone(&self.payload),
                _align: ::core::clone::Clone::clone(&self._align),
              }
            }
          }
          #[automatically_derived]
          impl<T0: ::core::marker::Copy + Copy, T1: ::core::marker::Copy + Copy> ::core::marker::Copy
            for LowerInvocation<T0, T1>
          {
          }
          unsafe impl wasmtime::component::ComponentType for Invocation {
            type Lower = LowerInvocation<
              <Operation as wasmtime::component::ComponentType>::Lower,
              <Json as wasmtime::component::ComponentType>::Lower,
            >;
            const ABI: wasmtime::component::__internal::CanonicalAbiInfo =
              wasmtime::component::__internal::CanonicalAbiInfo::record_static(&[
                <Operation as wasmtime::component::ComponentType>::ABI,
                <Json as wasmtime::component::ComponentType>::ABI,
              ]);
            #[inline]
            fn typecheck(
              ty: &wasmtime::component::__internal::InterfaceType,
              types: &wasmtime::component::__internal::InstanceType<'_>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
              wasmtime::component::__internal::typecheck_record(
                ty,
                types,
                &[
                  ("target", <Operation as wasmtime::component::ComponentType>::typecheck),
                  ("payload", <Json as wasmtime::component::ComponentType>::typecheck),
                ],
              )
            }
          }
        };
        impl core::fmt::Debug for Invocation {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Invocation")
              .field("target", &self.target)
              .field("payload", &self.payload)
              .finish()
          }
        }
        const _: () = {
          if !(24 == <Invocation as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 24 == <Invocation as wasmtime::component::ComponentType>::SIZE32",)
          }
          if !(4 == <Invocation as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 4 == <Invocation as wasmtime::component::ComponentType>::ALIGN32",)
          }
        };
        pub struct Response {
          pub payload: Json,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Response {
          #[inline]
          fn clone(&self) -> Response {
            Response {
              payload: ::core::clone::Clone::clone(&self.payload),
            }
          }
        }
        unsafe impl wasmtime::component::Lower for Response {
          #[inline]
          fn lower<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            dst: &mut std::mem::MaybeUninit<Self::Lower>,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::lower(&self.payload, cx, ty.fields[0usize].ty, {
              #[allow(unused_unsafe)]
              {
                unsafe {
                  use ::wasmtime::component::__internal::MaybeUninitExt;
                  let m: &mut std::mem::MaybeUninit<_> = dst;
                  m.map(|p| &mut (*p).payload)
                }
              }
            })?;
            Ok(())
          }
          #[inline]
          fn store<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            mut offset: usize,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            if true {
              if !(offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                  "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                )
              }
            }
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            wasmtime::component::Lower::store(
              &self.payload,
              cx,
              ty.fields[0usize].ty,
              <Json as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset),
            )?;
            Ok(())
          }
        }
        unsafe impl wasmtime::component::Lift for Response {
          #[inline]
          fn lift(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            src: &Self::Lower,
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            Ok(Self {
              payload: <Json as wasmtime::component::Lift>::lift(cx, ty.fields[0usize].ty, &src.payload)?,
            })
          }
          #[inline]
          fn load(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            bytes: &[u8],
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Record(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            if true {
              if !((bytes.as_ptr() as usize) % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                                      "assertion failed: (bytes.as_ptr() as usize) %\\n        (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                                  )
              }
            }
            let mut offset = 0;
            Ok(Self {
              payload: <Json as wasmtime::component::Lift>::load(
                cx,
                ty.fields[0usize].ty,
                &bytes[<Json as wasmtime::component::ComponentType>::ABI.next_field32_size(&mut offset)..]
                  [..<Json as wasmtime::component::ComponentType>::SIZE32],
              )?,
            })
          }
        }
        const _: () = {
          #[doc(hidden)]
          #[repr(C)]
          pub struct LowerResponse<T0: Copy> {
            payload: T0,
            _align: [wasmtime::ValRaw; 0],
          }
          #[automatically_derived]
          impl<T0: ::core::clone::Clone + Copy> ::core::clone::Clone for LowerResponse<T0> {
            #[inline]
            fn clone(&self) -> LowerResponse<T0> {
              LowerResponse {
                payload: ::core::clone::Clone::clone(&self.payload),
                _align: ::core::clone::Clone::clone(&self._align),
              }
            }
          }
          #[automatically_derived]
          impl<T0: ::core::marker::Copy + Copy> ::core::marker::Copy for LowerResponse<T0> {}
          unsafe impl wasmtime::component::ComponentType for Response {
            type Lower = LowerResponse<<Json as wasmtime::component::ComponentType>::Lower>;
            const ABI: wasmtime::component::__internal::CanonicalAbiInfo =
              wasmtime::component::__internal::CanonicalAbiInfo::record_static(&[
                <Json as wasmtime::component::ComponentType>::ABI,
              ]);
            #[inline]
            fn typecheck(
              ty: &wasmtime::component::__internal::InterfaceType,
              types: &wasmtime::component::__internal::InstanceType<'_>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
              wasmtime::component::__internal::typecheck_record(
                ty,
                types,
                &[("payload", <Json as wasmtime::component::ComponentType>::typecheck)],
              )
            }
          }
        };
        impl core::fmt::Debug for Response {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Response").field("payload", &self.payload).finish()
          }
        }
        const _: () = {
          if !(8 == <Response as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 8 == <Response as wasmtime::component::ComponentType>::SIZE32",)
          }
          if !(4 == <Response as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 4 == <Response as wasmtime::component::ComponentType>::ALIGN32",)
          }
        };
        pub enum InvocationError {
          Internal,
        }
        #[automatically_derived]
        impl ::core::marker::Copy for InvocationError {}
        #[automatically_derived]
        impl ::core::clone::Clone for InvocationError {
          #[inline]
          fn clone(&self) -> InvocationError {
            *self
          }
        }
        unsafe impl wasmtime::component::Lower for InvocationError {
          #[inline]
          fn lower<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            dst: &mut std::mem::MaybeUninit<Self::Lower>,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Variant(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            match self {
              Self::Internal => {
                {
                  #[allow(unused_unsafe)]
                  {
                    unsafe {
                      use ::wasmtime::component::__internal::MaybeUninitExt;
                      let m: &mut std::mem::MaybeUninit<_> = dst;
                      m.map(|p| &mut (*p).tag)
                    }
                  }
                }
                .write(wasmtime::ValRaw::u32(0u32));
                unsafe {
                  wasmtime::component::__internal::lower_payload(
                    {
                      #[allow(unused_unsafe)]
                      {
                        unsafe {
                          use ::wasmtime::component::__internal::MaybeUninitExt;
                          let m: &mut std::mem::MaybeUninit<_> = dst;
                          m.map(|p| &mut (*p).payload)
                        }
                      }
                    },
                    |payload| {
                      #[allow(unused_unsafe)]
                      {
                        unsafe {
                          use ::wasmtime::component::__internal::MaybeUninitExt;
                          let m: &mut std::mem::MaybeUninit<_> = payload;
                          m.map(|p| &mut (*p).Internal)
                        }
                      }
                    },
                    |dst| Ok(()),
                  )
                }
              }
            }
          }
          #[inline]
          fn store<T>(
            &self,
            cx: &mut wasmtime::component::__internal::LowerContext<'_, T>,
            ty: wasmtime::component::__internal::InterfaceType,
            mut offset: usize,
          ) -> wasmtime::component::__internal::anyhow::Result<()> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Variant(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            if true {
              if !(offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0) {
                panic!(
                  "assertion failed: offset % (<Self as wasmtime::component::ComponentType>::ALIGN32 as usize) == 0",
                )
              }
            }
            match self {
              Self::Internal => {
                *cx.get::<1usize>(offset) = 0u8.to_le_bytes();
                Ok(())
              }
            }
          }
        }
        unsafe impl wasmtime::component::Lift for InvocationError {
          #[inline]
          fn lift(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            src: &Self::Lower,
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Variant(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            Ok(match src.tag.get_u32() {
              0u32 => Self::Internal,
              discrim => {
                anyhow::bail!("unexpected discriminant: {0}", discrim);
              }
            })
          }
          #[inline]
          fn load(
            cx: &mut wasmtime::component::__internal::LiftContext<'_>,
            ty: wasmtime::component::__internal::InterfaceType,
            bytes: &[u8],
          ) -> wasmtime::component::__internal::anyhow::Result<Self> {
            let align = <Self as wasmtime::component::ComponentType>::ALIGN32;
            if true {
              if !((bytes.as_ptr() as usize) % (align as usize) == 0) {
                panic!("assertion failed: (bytes.as_ptr() as usize) % (align as usize) == 0")
              }
            }
            let discrim = bytes[0];
            let payload_offset = <Self as wasmtime::component::__internal::ComponentVariant>::PAYLOAD_OFFSET32;
            let payload = &bytes[payload_offset..];
            let ty = match ty {
              wasmtime::component::__internal::InterfaceType::Variant(i) => &cx.types[i],
              _ => wasmtime::component::__internal::bad_type_info(),
            };
            Ok(match discrim {
              0u8 => Self::Internal,
              discrim => {
                anyhow::bail!("unexpected discriminant: {0}", discrim);
              }
            })
          }
        }
        const _: () = {
          #[doc(hidden)]
          #[repr(C)]
          #[derive(Clone)]
          pub struct LowerInvocationError {
            tag: wasmtime::ValRaw,
            payload: LowerPayloadInvocationError,
          }
          #[automatically_derived]
          impl ::core::marker::Copy for LowerInvocationError {}
          #[doc(hidden)]
          #[allow(non_snake_case)]
          #[repr(C)]
          #[derive(Clone)]
          union LowerPayloadInvocationError {
            Internal: [wasmtime::ValRaw; 0],
          }
          #[automatically_derived]
          #[allow(non_snake_case)]
          impl ::core::marker::Copy for LowerPayloadInvocationError {}
          unsafe impl wasmtime::component::ComponentType for InvocationError {
            type Lower = LowerInvocationError;
            #[inline]
            fn typecheck(
              ty: &wasmtime::component::__internal::InterfaceType,
              types: &wasmtime::component::__internal::InstanceType<'_>,
            ) -> wasmtime::component::__internal::anyhow::Result<()> {
              wasmtime::component::__internal::typecheck_variant(ty, types, &[("internal", None)])
            }
            const ABI: wasmtime::component::__internal::CanonicalAbiInfo =
              wasmtime::component::__internal::CanonicalAbiInfo::variant_static(&[None]);
          }
          unsafe impl wasmtime::component::__internal::ComponentVariant for InvocationError {
            const CASES: &'static [Option<wasmtime::component::__internal::CanonicalAbiInfo>] = &[None];
          }
        };
        impl core::fmt::Debug for InvocationError {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
              InvocationError::Internal => f.debug_tuple("InvocationError::Internal").finish(),
            }
          }
        }
        impl core::fmt::Display for InvocationError {
          fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("{0:?}", self))
          }
        }
        impl std::error::Error for InvocationError {}
        const _: () = {
          if !(1 == <InvocationError as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 1 == <InvocationError as wasmtime::component::ComponentType>::SIZE32",)
          }
          if !(1 == <InvocationError as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 1 == <InvocationError as wasmtime::component::ComponentType>::ALIGN32",)
          }
        };
        pub trait Host {
          #[must_use]
          #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
          fn request_sync<'life0, 'async_trait>(
            &'life0 mut self,
            invocation: Invocation,
          ) -> ::core::pin::Pin<
            Box<
              dyn ::core::future::Future<Output = wasmtime::Result<Result<Response, InvocationError>>>
                + ::core::marker::Send
                + 'async_trait,
            >,
          >
          where
            'life0: 'async_trait,
            Self: 'async_trait;
          #[must_use]
          #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
          fn request_async<'life0, 'async_trait>(
            &'life0 mut self,
            invocation: Invocation,
          ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output = wasmtime::Result<u64>> + ::core::marker::Send + 'async_trait>,
          >
          where
            'life0: 'async_trait,
            Self: 'async_trait;
          #[must_use]
          #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
          fn get_response<'life0, 'async_trait>(
            &'life0 mut self,
            id: u64,
          ) -> ::core::pin::Pin<
            Box<
              dyn ::core::future::Future<Output = wasmtime::Result<Result<Response, InvocationError>>>
                + ::core::marker::Send
                + 'async_trait,
            >,
          >
          where
            'life0: 'async_trait,
            Self: 'async_trait;
          #[must_use]
          #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
          fn cancel_request<'life0, 'async_trait>(
            &'life0 mut self,
            id: u64,
          ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output = wasmtime::Result<()>> + ::core::marker::Send + 'async_trait>,
          >
          where
            'life0: 'async_trait,
            Self: 'async_trait;
        }
        pub fn add_to_linker<T, U>(
          linker: &mut wasmtime::component::Linker<T>,
          get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
          T: Send,
          U: Host + Send,
        {
          let mut inst = linker.instance("candle:wick/wick")?;
          inst.func_wrap_async(
            "request-sync",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (Invocation,)| {
              Box::new(async move {
                let host = get(caller.data_mut());
                let r = Host::request_sync(host, arg0).await;
                Ok((r?,))
              })
            },
          )?;
          inst.func_wrap_async(
            "request-async",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (Invocation,)| {
              Box::new(async move {
                let host = get(caller.data_mut());
                let r = Host::request_async(host, arg0).await;
                Ok((r?,))
              })
            },
          )?;
          inst.func_wrap_async(
            "get-response",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (u64,)| {
              Box::new(async move {
                let host = get(caller.data_mut());
                let r = Host::get_response(host, arg0).await;
                Ok((r?,))
              })
            },
          )?;
          inst.func_wrap_async(
            "cancel-request",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (u64,)| {
              Box::new(async move {
                let host = get(caller.data_mut());
                let r = Host::cancel_request(host, arg0).await;
                r
              })
            },
          )?;
          Ok(())
        }
      }
    }
  }
  const _: &str = "interface wick {\n  type json = string\n\n  record invocation {\n    target: operation,\n    payload: json,\n  }\n\n  record operation {\n    name: string,\n    component: string,\n  }\n\n  record response {\n    payload: json,\n  }\n\n  variant invocation-error {\n    internal,\n  }\n\n  request-sync: func(invocation: invocation) -> result<response, invocation-error>\n\n  request-async: func(invocation: invocation) -> u64\n  get-response: func(id: u64) -> result<response, invocation-error>\n  cancel-request: func(id: u64) -> ()\n}\n\n";
  const _: &str =
    "package candle:wick\n\nworld command-trigger {\n  import wick\n}\n\nworld component {\n  import wick\n}";
  pub(crate) use candle::wick::wick::*;
}
#[allow(unused)]
impl generated::Host for crate::component::state::ComponentState {
  #[allow(
    clippy::async_yields_async,
    clippy::diverging_sub_expression,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
  )]
  fn request_sync<'life0, 'async_trait>(
    &'life0 mut self,
    invocation: generated::Invocation,
  ) -> ::core::pin::Pin<
    Box<
      dyn ::core::future::Future<Output = wasmtime::Result<Result<generated::Response, generated::InvocationError>>>
        + ::core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    Box::pin(async move {
      if let ::core::option::Option::Some(__ret) =
        ::core::option::Option::None::<wasmtime::Result<Result<generated::Response, generated::InvocationError>>>
      {
        return __ret;
      }
      let mut __self = self;
      let invocation = invocation;
      let __ret: wasmtime::Result<Result<generated::Response, generated::InvocationError>> =
        { panic!("not yet implemented") };
      #[allow(unreachable_code)]
      __ret
    })
  }
  #[allow(
    clippy::async_yields_async,
    clippy::diverging_sub_expression,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
  )]
  fn request_async<'life0, 'async_trait>(
    &'life0 mut self,
    invocation: generated::Invocation,
  ) -> ::core::pin::Pin<
    Box<dyn ::core::future::Future<Output = wasmtime::Result<u64>> + ::core::marker::Send + 'async_trait>,
  >
  where
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    Box::pin(async move {
      if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<wasmtime::Result<u64>> {
        return __ret;
      }
      let mut __self = self;
      let invocation = invocation;
      let __ret: wasmtime::Result<u64> = { panic!("not yet implemented") };
      #[allow(unreachable_code)]
      __ret
    })
  }
  #[allow(
    clippy::async_yields_async,
    clippy::diverging_sub_expression,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
  )]
  fn get_response<'life0, 'async_trait>(
    &'life0 mut self,
    id: u64,
  ) -> ::core::pin::Pin<
    Box<
      dyn ::core::future::Future<Output = wasmtime::Result<Result<generated::Response, generated::InvocationError>>>
        + ::core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    Box::pin(async move {
      if let ::core::option::Option::Some(__ret) =
        ::core::option::Option::None::<wasmtime::Result<Result<generated::Response, generated::InvocationError>>>
      {
        return __ret;
      }
      let mut __self = self;
      let id = id;
      let __ret: wasmtime::Result<Result<generated::Response, generated::InvocationError>> =
        { panic!("not yet implemented") };
      #[allow(unreachable_code)]
      __ret
    })
  }
  #[allow(
    clippy::async_yields_async,
    clippy::diverging_sub_expression,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
  )]
  fn cancel_request<'life0, 'async_trait>(
    &'life0 mut self,
    id: u64,
  ) -> ::core::pin::Pin<
    Box<dyn ::core::future::Future<Output = wasmtime::Result<()>> + ::core::marker::Send + 'async_trait>,
  >
  where
    'life0: 'async_trait,
    Self: 'async_trait,
  {
    Box::pin(async move {
      if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<wasmtime::Result<()>> {
        return __ret;
      }
      let mut __self = self;
      let id = id;
      let __ret: wasmtime::Result<()> = { panic!("not yet implemented") };
      #[allow(unreachable_code)]
      __ret
    })
  }
}
