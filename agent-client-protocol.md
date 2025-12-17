# Crate Documentation

**Version:** 0.9.0

**Format Version:** 57

# Module `agent_client_protocol`

## Types

### Struct `ClientSideConnection`

A client-side connection to an agent.

This struct provides the client's view of an ACP connection, allowing
clients (such as code editors) to communicate with agents. It implements
the [`Agent`] trait to provide methods for initializing sessions, sending
prompts, and managing the agent lifecycle.

See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)

```rust
pub struct ClientSideConnection {
    // Some fields omitted
}
```

#### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

#### Implementations

##### Methods

- ```rust
  pub fn new</* synthetic */ impl MessageHandler<ClientSide> + 'static: MessageHandler<ClientSide> + ''static, /* synthetic */ impl Unpin + AsyncWrite: Unpin + AsyncWrite, /* synthetic */ impl Unpin + AsyncRead: Unpin + AsyncRead, /* synthetic */ impl Fn(LocalBoxFuture<'static, ()>) + 'static: Fn(LocalBoxFuture<''static, ()>) + ''static>(client: impl MessageHandler<ClientSide> + ''static, outgoing_bytes: impl Unpin + AsyncWrite, incoming_bytes: impl Unpin + AsyncRead, spawn: impl Fn(LocalBoxFuture<''static, ()>) + ''static) -> (Self, impl Future<Output = Result<()>>) { /* ... */ }
  ```
  Creates a new client-side connection to an agent.

- ```rust
  pub fn subscribe(self: &Self) -> StreamReceiver { /* ... */ }
  ```
  Subscribe to receive stream updates from the agent.

##### Trait Implementations

- **Agent**
  - ```rust
    fn initialize<''life0, ''async_trait>(self: &''life0 Self, args: InitializeRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<InitializeResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn authenticate<''life0, ''async_trait>(self: &''life0 Self, args: AuthenticateRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<AuthenticateResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn new_session<''life0, ''async_trait>(self: &''life0 Self, args: NewSessionRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<NewSessionResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn load_session<''life0, ''async_trait>(self: &''life0 Self, args: LoadSessionRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<LoadSessionResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn set_session_mode<''life0, ''async_trait>(self: &''life0 Self, args: SetSessionModeRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<SetSessionModeResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn prompt<''life0, ''async_trait>(self: &''life0 Self, args: PromptRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<PromptResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn cancel<''life0, ''async_trait>(self: &''life0 Self, args: CancelNotification) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<()>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn ext_method<''life0, ''async_trait>(self: &''life0 Self, args: ExtRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<ExtResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn ext_notification<''life0, ''async_trait>(self: &''life0 Self, args: ExtNotification) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<()>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **IntoOption**
  - ```rust
    fn into_option(self: Self) -> Option<T> { /* ... */ }
    ```

- **MessageHandler**
  - ```rust
    async fn handle_request(self: &Self, request: ClientRequest) -> Result<AgentResponse, Error> { /* ... */ }
    ```

  - ```rust
    async fn handle_notification(self: &Self, notification: ClientNotification) -> Result<(), Error> { /* ... */ }
    ```

- **RefUnwindSafe**
- **Send**
- **Sync**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
### Struct `ClientSide`

Marker type representing the client side of an ACP connection.

This type is used by the RPC layer to determine which messages
are incoming vs outgoing from the client's perspective.

See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)

```rust
pub struct ClientSide;
```

#### Implementations

##### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ClientSide { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **DynClone**
  - ```rust
    fn __clone_box(self: &Self, _: Private) -> *mut () { /* ... */ }
    ```

- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **IntoOption**
  - ```rust
    fn into_option(self: Self) -> Option<T> { /* ... */ }
    ```

- **MessageHandler**
  - ```rust
    async fn handle_request(self: &Self, request: AgentRequest) -> Result<ClientResponse, Error> { /* ... */ }
    ```

  - ```rust
    async fn handle_notification(self: &Self, notification: AgentNotification) -> Result<(), Error> { /* ... */ }
    ```

- **RefUnwindSafe**
- **Send**
- **Side**
  - ```rust
    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<AgentRequest> { /* ... */ }
    ```

  - ```rust
    fn decode_notification(method: &str, params: Option<&RawValue>) -> Result<AgentNotification> { /* ... */ }
    ```

- **Sync**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
### Struct `AgentSideConnection`

An agent-side connection to a client.

This struct provides the agent's view of an ACP connection, allowing
agents to communicate with clients. It implements the [`Client`] trait
to provide methods for requesting permissions, accessing the file system,
and sending session updates.

See protocol docs: [Agent](https://agentclientprotocol.com/protocol/overview#agent)

```rust
pub struct AgentSideConnection {
    // Some fields omitted
}
```

#### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

#### Implementations

##### Methods

- ```rust
  pub fn new</* synthetic */ impl MessageHandler<AgentSide> + 'static: MessageHandler<AgentSide> + ''static, /* synthetic */ impl Unpin + AsyncWrite: Unpin + AsyncWrite, /* synthetic */ impl Unpin + AsyncRead: Unpin + AsyncRead, /* synthetic */ impl Fn(LocalBoxFuture<'static, ()>) + 'static: Fn(LocalBoxFuture<''static, ()>) + ''static>(agent: impl MessageHandler<AgentSide> + ''static, outgoing_bytes: impl Unpin + AsyncWrite, incoming_bytes: impl Unpin + AsyncRead, spawn: impl Fn(LocalBoxFuture<''static, ()>) + ''static) -> (Self, impl Future<Output = Result<()>>) { /* ... */ }
  ```
  Creates a new agent-side connection to a client.

- ```rust
  pub fn subscribe(self: &Self) -> StreamReceiver { /* ... */ }
  ```
  Subscribe to receive stream updates from the client.

##### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Client**
  - ```rust
    fn request_permission<''life0, ''async_trait>(self: &''life0 Self, args: RequestPermissionRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<RequestPermissionResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn write_text_file<''life0, ''async_trait>(self: &''life0 Self, args: WriteTextFileRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<WriteTextFileResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn read_text_file<''life0, ''async_trait>(self: &''life0 Self, args: ReadTextFileRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<ReadTextFileResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn create_terminal<''life0, ''async_trait>(self: &''life0 Self, args: CreateTerminalRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<CreateTerminalResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn terminal_output<''life0, ''async_trait>(self: &''life0 Self, args: TerminalOutputRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<TerminalOutputResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn release_terminal<''life0, ''async_trait>(self: &''life0 Self, args: ReleaseTerminalRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<ReleaseTerminalResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn wait_for_terminal_exit<''life0, ''async_trait>(self: &''life0 Self, args: WaitForTerminalExitRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<WaitForTerminalExitResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn kill_terminal_command<''life0, ''async_trait>(self: &''life0 Self, args: KillTerminalCommandRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<KillTerminalCommandResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn session_notification<''life0, ''async_trait>(self: &''life0 Self, args: SessionNotification) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<()>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn ext_method<''life0, ''async_trait>(self: &''life0 Self, args: ExtRequest) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<ExtResponse>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

  - ```rust
    fn ext_notification<''life0, ''async_trait>(self: &''life0 Self, args: ExtNotification) -> ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<()>> + ''async_trait>>
where
    Self: ''async_trait,
    ''life0: ''async_trait { /* ... */ }
    ```

- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **IntoOption**
  - ```rust
    fn into_option(self: Self) -> Option<T> { /* ... */ }
    ```

- **MessageHandler**
  - ```rust
    async fn handle_request(self: &Self, request: AgentRequest) -> Result<ClientResponse, Error> { /* ... */ }
    ```

  - ```rust
    async fn handle_notification(self: &Self, notification: AgentNotification) -> Result<(), Error> { /* ... */ }
    ```

- **RefUnwindSafe**
- **Send**
- **Sync**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
### Struct `AgentSide`

Marker type representing the agent side of an ACP connection.

This type is used by the RPC layer to determine which messages
are incoming vs outgoing from the agent's perspective.

See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)

```rust
pub struct AgentSide;
```

#### Implementations

##### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> AgentSide { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **DynClone**
  - ```rust
    fn __clone_box(self: &Self, _: Private) -> *mut () { /* ... */ }
    ```

- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **IntoOption**
  - ```rust
    fn into_option(self: Self) -> Option<T> { /* ... */ }
    ```

- **MessageHandler**
  - ```rust
    async fn handle_request(self: &Self, request: ClientRequest) -> Result<AgentResponse, Error> { /* ... */ }
    ```

  - ```rust
    async fn handle_notification(self: &Self, notification: ClientNotification) -> Result<(), Error> { /* ... */ }
    ```

- **RefUnwindSafe**
- **Send**
- **Side**
  - ```rust
    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<ClientRequest> { /* ... */ }
    ```

  - ```rust
    fn decode_notification(method: &str, params: Option<&RawValue>) -> Result<ClientNotification> { /* ... */ }
    ```

- **Sync**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
## Re-exports

### Re-export `StreamMessage`

```rust
pub use stream_broadcast::StreamMessage;
```

### Re-export `StreamMessageContent`

```rust
pub use stream_broadcast::StreamMessageContent;
```

### Re-export `StreamMessageDirection`

```rust
pub use stream_broadcast::StreamMessageDirection;
```

### Re-export `StreamReceiver`

```rust
pub use stream_broadcast::StreamReceiver;
```

### Re-export `agent::*`

```rust
pub use agent::*;
```

### Re-export `agent_client_protocol_schema::*`

```rust
pub use agent_client_protocol_schema::*;
```

### Re-export `client::*`

```rust
pub use client::*;
```

### Re-export `rpc::*`

```rust
pub use rpc::*;
```

