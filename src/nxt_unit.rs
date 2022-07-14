#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// nxt_unit_sptr_set and nxt_unit_sptr_get need manual translations as bindgen
// cannot translate inline functions with code, only declarations.

// static inline void
// nxt_unit_sptr_set(nxt_unit_sptr_t *sptr, void *ptr)
// {
//     sptr->offset = (uint8_t *) ptr - sptr->base;
// }

#[inline]
pub unsafe extern "C" fn nxt_unit_sptr_set(
    sptr: *mut nxt_unit_sptr_t,
    ptr: *mut ::std::os::raw::c_void,
) {
    let origin = (*sptr).base.as_ptr();
    (*sptr).offset = (ptr as *mut u8).offset_from(origin) as u32;
}

// static inline void *
// nxt_unit_sptr_get(nxt_unit_sptr_t *sptr)
// {
//     return sptr->base + sptr->offset;
// }

#[inline]
pub unsafe extern "C" fn nxt_unit_sptr_get(
    sptr: *const nxt_unit_sptr_t,
) -> *mut ::std::os::raw::c_void {
    (*sptr).base.as_ptr().offset((*sptr).offset as isize) as *mut ::std::os::raw::c_void
}
