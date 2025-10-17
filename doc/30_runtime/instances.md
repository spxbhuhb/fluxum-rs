# Instance subsystem

The instance subsystem manages fragment instances.

```rust
pub type InstanceKey = Index;

trait InstanceRuntime {

}

struct InstanceRuntimeImpl {
    instances: Arena<FragmentInst>
}

pub struct FragmentInst {
    pub id: u32,
    pub desc: &'static FragmentDesc,
    pub internal_stores: smallvec::SmallVec<[StoreKey; 8]>, // used for cleanup when the instance is dropped
    pub subscriptions: smallvec::SmallVec<[SubscriptionKey; 8]>, // used for cleanup when the instance is dropped
    pub children: smallvec::SmallVec<[FragmentInst; 8]> // used for cleanup when the instance is dropped
}
```