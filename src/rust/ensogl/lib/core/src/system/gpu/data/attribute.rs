//! This module defines attributes and related utilities.

use crate::prelude::*;

use crate::control::callback::CallbackFn;
use crate::data::OptVec;
use crate::data::dirty;
use crate::debug::Stats;
use crate::system::gpu::Context;

use crate::data::dirty::traits::*;
use crate::system::gpu::types::*;

use std::collections::BTreeSet;



// =============
// === Types ===
// =============

newtype_copy! {
    /// Index of the attribute instance.
    AttributeInstanceIndex(usize);

    /// Index of the attribute instance.
    BufferIndex(usize);
}

/// Dirty flag collecting information which buffers were mutated.
pub type BufferDirty = dirty::SharedBitField<u64,Box<dyn Fn()>>;

/// Dirty flag indicating that the shape of the attribute (all buffers) was changed.
pub type ShapeDirty = dirty::SharedBool<Box<dyn Fn()>>;



// ======================
// === AttributeScope ===
// ======================

shared! { AttributeScope
/// Scope defines a view for geometry structure. For example, there is point scope or instance
/// scope. Scope contains buffer of data for each item it describes.
///
/// # Memory management and ID re-use.
/// TODO: The proper memory management should be implemented. Currently, after creating a lot of
///       instances and dropping them, the memory is not freed. This section explains why and
///       describes possible solutions.
///
/// Currently, the `free_ids` field keeps track of all instance indexes that are not used anymore.
/// It is stored in a sorted container in order to preserve the display order when creating new
/// instances (new instances appear on top of older instances if no dropping happened in the
/// meantime). Alternative sorting solutions are described in the docs for the [`Symbol`].
///
/// In order to properly re-use memory, we need to free it sometimes. The following solutions are
/// possible:
///
/// 1. Keeping track of all free indexes in a sorted container (like [`BTreeSet`] or the specialized
///    [`enso_data::Diet`] and in case the biggest index is freed, iterating over the indexes and
///    freeing as much as possible. This solution has the downside that the indexes are stored in
///    order, so insertion and deletion is much slower than when using unordered [`Vec`]. Also, this
///    does not work well if a instance with a big ID is kept alive, as it will prevent memory of
///    all instances with smaller IDs from being cleaned. See benchmarks in the `enso_data::diet`
///    module to learn more.
///
/// 2. Keeping track of all free indexes in an unordered container and in case the biggest index is
///    freed, sorting the container and freeing the memory. As an optimization, the sorting might
///    be performed after the frame (or several frames) was drawn. It's not obvious when this
///    solution will be slower / faster than the solution (1), but time differences may be big.
///    See benchmarks in the `enso_data::diet` module to learn more.
///
/// 3. Keeping track of all free indexes and in case a lot of them are free, re-ordering the
///    instances and freeing the memory. This would require all instance-users (like [`Sprite`]s) to
///    keep instance IDs in some kind of `Rc<Cell<ID>>`, which may slow attrib read/write down.
///    However, this solution works well even if an instance with a big ID is kept alive. It's not
///    obvious when this solution will be slower / faster than other ones, but time differences may
///    be big. See benchmarks in the `enso_data::diet` module to learn more.
///
/// To learn more about these mechanisms and connected design decisions, read the docs of
/// [`Symbol`], especially the "Changing attribute & GPU memory consumption" sections.

#[derive(Debug)]
pub struct AttributeScopeData {
    buffers         : OptVec<AnyBuffer>,
    buffer_dirty    : BufferDirty,
    shape_dirty     : ShapeDirty,
    buffer_name_map : HashMap<String,BufferIndex>,
    logger          : Logger,
    free_ids        : BTreeSet<AttributeInstanceIndex>,
    size            : usize,
    context         : Context,
    stats           : Stats,
}

impl {
    /// Create a new scope with the provided dirty callback.
    pub fn new<OnMut:CallbackFn+Clone>
    (lgr:Logger, stats:&Stats, context:&Context, on_mut:OnMut) -> Self {
        info!(lgr,"Initializing.",|| {
            let logger          = lgr.clone();
            let stats           = stats.clone_ref();
            let buffer_logger   = Logger::sub(&logger,"buffer_dirty");
            let shape_logger    = Logger::sub(&logger,"shape_dirty");
            let buffer_dirty    = BufferDirty::new(buffer_logger,Box::new(on_mut.clone()));
            let shape_dirty     = ShapeDirty::new(shape_logger,Box::new(on_mut));
            let buffers         = default();
            let buffer_name_map = default();
            let free_ids        = default();
            let size            = default();
            let context         = context.clone();
            Self {context,buffers,buffer_dirty,shape_dirty,buffer_name_map,logger,free_ids,size
                 ,stats}
        })
    }

    /// Add a new named buffer to the scope.
    pub fn add_buffer<Name:Str, T:Storable>(&mut self, name:Name) -> Buffer<T>
    where AnyBuffer: From<Buffer<T>> {
        let name         = name.as_ref().to_string();
        let buffer_dirty = self.buffer_dirty.clone();
        let shape_dirty  = self.shape_dirty.clone();
        let ix           = self.buffers.reserve_index();
        debug!(self.logger, "Adding buffer '{name}' at index {ix}.", || {
            let on_set     = Box::new(move || { buffer_dirty.set(ix) });
            let on_resize  = Box::new(move || { shape_dirty.set() });
            let logger     = Logger::sub(&self.logger,&name);
            let buffer     = Buffer::new(logger,&self.stats,&self.context,on_set,on_resize);
            let buffer_ref = buffer.clone();
            self.buffers.set(ix,AnyBuffer::from(buffer));
            self.buffer_name_map.insert(name,ix.into());
            self.shape_dirty.set();
            buffer_ref
        })
    }

    /// Lookup buffer by a given name.
    pub fn buffer(&self, name:&str) -> Option<AnyBuffer> {
        self.buffer_name_map.get(name).map(|i| self.buffers[(*i).into()].clone())
    }

    /// Checks if a buffer with the given name was created in this scope.
    pub fn contains<S:Str>(&self, name:S) -> bool {
        self.buffer_name_map.contains_key(name.as_ref())
    }

    /// Add a new instance to every buffer in the scope.
    pub fn add_instance(&mut self) -> AttributeInstanceIndex {
        let instance_count = 1;
        debug!(self.logger, "Adding {instance_count} instance(s).", || {
            match self.free_ids.iter().next().copied() {
                Some(ix) => {
                    self.free_ids.remove(&ix);
                    ix
                }
                None => {
                    let ix = self.size;
                    self.size += instance_count;
                    self.buffers.iter_mut().for_each(|t| t.add_element());
                    ix.into()
                }
            }
        })
    }

    /// Dispose instance for reuse in the future. All data in all buffers at the provided `id` will
    /// be set to default.
    pub fn dispose(&mut self, id:AttributeInstanceIndex) {
        debug!(self.logger, "Disposing instance {id}.", || {
            for buffer in &self.buffers {
                buffer.set_to_default(id.into())
            }
            self.free_ids.insert(id);
        })
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        debug!(self.logger, "Updating.", || {
            if self.shape_dirty.check() {
                for i in 0..self.buffers.len() {
                    self.buffers[i].update()
                }
            } else {
                for i in 0..self.buffers.len() {
                    if self.buffer_dirty.check(&i) {
                        self.buffers[i].update()
                    }
                }
            }
            self.shape_dirty.unset();
            self.buffer_dirty.unset_all();
        })
    }

    /// Return the size of buffers in this scope.
    pub fn size(&self) -> usize {
        self.size
    }
}}



// =================
// === Attribute ===
// =================

/// View for a particular buffer. Allows reading and writing buffer data
/// via the internal mutability pattern. It is implemented as a view on
/// a selected `Buffer` element under the hood.
#[derive(Clone,CloneRef,Debug,Derivative)]
pub struct Attribute<T> {
    index  : AttributeInstanceIndex,
    buffer : Buffer<T>
}

impl<T> Attribute<T> {
    /// Create a new variable as an indexed view over provided buffer.
    pub fn new(index:AttributeInstanceIndex, buffer:Buffer<T>) -> Self {
        Self {index,buffer}
    }
}

impl<T:Storable> Attribute<T> {
    /// Get a copy of the data this attribute points to.
    pub fn get(&self) -> T {
        self.buffer.get(self.index.into())
    }

    /// Set the data this attribute points to.
    pub fn set(&self, value:T) {
        self.buffer.set(self.index.into(),value);
    }

    /// Modify the data this attribute points to.
    pub fn modify<F:FnOnce(&mut T)>(&self, f:F) {
        let mut value = self.get();
        f(&mut value);
        self.set(value);
    }
}
