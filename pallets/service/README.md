# **pallet-service**
## **Data structures**
### **Order**
User specified data about the service
```rust
    pub struct Order {
        /// Service data.
        pub binary: Vec<u8>,
        /// Service dns.
        pub dns: Vec<u8>,
        /// Service name.
        pub name: Option<Vec<u8>>,
        /// duration to be served, none means run endless until removed
        pub duration: BlockNumber,
        /// maximum number of geodes to serve the order
        pub geode_num: u32,
    }
```
### **Service**
Status of a service on chain
```rust
    pub enum ServiceState {
        /// Default state, the service not existing
        Null,
        /// Waiting for geode to serve the service.
        Pending,
        /// When the service is being serviced by geode.
        Online,
        /// When no geode is serving after the service online.
        Offline,
        /// When the service is completed or cancelled by user
        Terminated,
    }

    pub struct Service<AccountId: Ord, Hash> {
        /// Service order id, identitcal to `service_id`
        pub order_id: Hash,
        /// Current existing dispatch for this service
        pub dispatches: BTreeSet<DispatchId>,
        /// Service owner id.
        pub owner: AccountId,
        /// Geodes serving the service(already put online).
        pub geodes: BTreeSet<AccountId>,
        /// Total weighted uptime the service has been online (num of geode * online block num)
        pub weighted_uptime: u64,
        /// Expected block num for the service to complete
        pub expected_ending: Option<BlockNumber>,
        /// Whether the service has backup
        pub backup_flag: bool,
        /// Indexing for backups, key is the backup service id, value is the backup data hash
        pub backup_map: BTreeMap<AccountId, Hash>,
        /// Current state of the service
        pub state: ServiceState,
    }
```
### **Dispatch**
Multiple dispatches will be generated regarding to the geode number specified in the service order, each dispatch will be mapped to one geode, and used as a reference for onboarding the geode for the service
```rust
    pub enum DispatchState {
        /// default state, the dispatch not exist
        Null,
        /// Pending to get a geode to query
        Pending,
        /// Waiting confirmation from geode
        Awaiting,
        /// Waiting dispatched geode to put online
        PreOnline,
    }

    pub struct Dispatch<AccountId: Ord, Hash> {
        /// DispatchId is incremental from 0 and updated by 1 whenever a new dispatch is generated, it ensures dispatches will be served based on FIFO order.
        pub dispatch_id: u32,
        /// The service_id for which this dispatch is generated 
        pub service_id: Hash,
        /// Geode assigned with this dispatch, None if no geode has been queried for this dispatch
        pub geode: Option<AccountId>,
        /// Dispatch state
        pub state: DispatchState,
    }
```


## **Interfaces**
### **Storage**
#### **Order related**
##### **`Orders`**
- **Description**: Storage of order struct
- **Map**: `order_id: Hash` => `Order` 

#### **Service related**
##### **`Services`**
- **Description**: Storage of service struct
- **Map**: `order_id: Hash` => `Service`
##### **`OnlineServices`**
- **Description**: services in `Online` state and when changed to this state, **will also be updated when the number of serving geode has changed**
- **Map**: `order_id: Hash` => `BlockNumber`
##### **`TerminatedBatch`**
- **Description**: Services get terminated in certain block
- **Map**: `BlockNumber` => `Services: BTreeSet<Hash>`
##### **`ExpectedEndings`**
- **Description**: Services expected to be ended at certain block number
- **Map**: `BlockNumber` => `Services: BTreeSet<Hash>`

#### **Dispatch related**
##### **`LatestDispatchId`**
- **Description**: The dispatch id of the latest generated dispatch
- **Value**: `u32`
##### **`Dispatches`**
- **Description**: 
- **Map**: `dispatch_id: u32` => `Dispatch`
##### **`PendingDispatchesQueue`**
- **Description**: Queue of dispatches pending to be assigned to query a geode
- **Map**: `dispatch_id: u32` => `order_id: Hash`
##### **`AwaitingDispatches`**
- **Description**: Information for dispatch waiting for geode's confirmation, including order_id, block number of when started waiting confirmation, and dispatch_id
- **Map**: `geode_id: AccountId` => `(order_id: Hash, started: BlockNumber, dispatch_id: u32)`
##### **`PreOnlineDispatches`**
- **Description**: Information for dispatch waiting for geode's confirmation, including order_id, block number of when started waiting online, and dispatch_id
- **Map**: `geode_id: AccountId` => `(order_id: Hash, started: BlockNumber, dispatch_id: u32)`

### **Extrinsics**
#### **`user_create_service`**
- **Parameters**: `service_order: Order`
- **Description**: Called by user to create a service order, new dispatches will also be generated correspondingly

#### **`user_remove_service`**
- **Parameters**: `service_id: Hash`
- **Description**: Called by user to remove a service order, will remove all available dispatches correspondinly

#### **`provider_confirm_dispatch`**
- **Parameters**: `geode: AccountId`, `service_id: Hash`
- **Description**: Called by provider of attested geode to confirm a dispatch for serving an order, currently geode is not allowed to reject as the dispatch is assigned based on the promise of geode, if the provider failed to confirm in time, the geode will be transitted to Unknown state. Geode will be transitted to Instantiated state after confirmation

#### **`provider_start_serving`**
- **Parameters**: `geode: AccountId, service_id: Hash`
- **Description**: Called by provider of instantiated geode to put the service online, the provider must ensure the service has been installed on the geode before calling this extrinsic, as the geode will undergo service liveness check after start serving. If the provider failed to confirm in time, the geode will be transitted to Unknown state.

#### **`provider_uninstantiate_geode`**
- **Parameters**: `geode: AccountId`
- **Description**: Called by provider of instantiated geode if the serving order has been terminated, provider must ensure the service has been uninstalled before calling this extrinsic, as the geode will be assigned with new dispatch afterward.

### **API**
NA

## **Workflows**


## **TODO list**
- Graceful termination of geode
- Service liveness check
- Invalid service installation
- Tokenomics integration

