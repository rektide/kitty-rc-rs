# Plan: Kitty Registry Connection Pool

## Problem Statement

The current `kitty-focus-tracker` implementation has several limitations:

1. **No connection pooling** - Each time we need to adjust font size, we create a new connection, authenticate, send command, and close the connection. This is inefficient and adds latency.

2. **No retry logic** - Transient connection failures cause the operation to fail immediately with no recovery.

3. **No status tracking** - We don't track which kitty instances are actually connected/controllable, leading to repeated failed attempts.

4. **Focus lost affects all kitties** - Currently, when focus is lost, we iterate through ALL kitty windows to decrease font size. This is incorrect - we should only affect the kitty that just lost focus.

## Proposed Solution

### Architecture

```rust
pub struct KittyRegistry {
    // Map of PID to managed connection
    connections: HashMap<i32, ManagedConnection>,

    // Track kitty remote control capability status
    status: HashMap<i32, KittyConnectionStatus>,

    // Global configuration
    config: RegistryConfig,
}

struct ManagedConnection {
    client: KittyClient,
    last_used: Instant,
    pid: i32,
    is_healthy: bool,
}

enum KittyConnectionStatus {
    NotChecked,
    Ready, // Socket exists and accepts commands
    NoSocket,    // Socket file doesn't exist
    Failed(String), // Connection failed with error
}
```

### 1. Connection Pooling

**Initialization:**

```rust
impl KittyRegistry {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            status: HashMap::new(),
            config: RegistryConfig::default(),
        }
    }

    // Get or create connection for a kitty PID
    pub async fn get_connection(&mut self, pid: i32) -> Result<&mut KittyClient, KittyError> {
        // Check status first
        self.check_status(pid).await;

        match self.status.get(&pid) {
            Some(KittyConnectionStatus::Ready) => {
                // Reuse existing connection
                if let Some(conn) = self.connections.get_mut(&pid) {
                    conn.last_used = Instant::now();
                    conn.is_healthy = true;
                    Ok(&mut conn.client)
                } else {
                    Err(KittyError::Connection(ConnectionError::ConnectionNotFound))
                }
            }
            _ => {
                // Try to establish new connection
                self.establish_connection(pid).await
            }
        }
    }

    async fn establish_connection(&mut self, pid: i32) -> Result<&mut KittyClient, KittyError> {
        let socket_path = get_kitty_socket_path(pid);

        // Attempt connection with timeout
        let client = KittyClient::connect_with_timeout(
            &socket_path,
            Duration::from_secs(self.config.connect_timeout_secs),
        ).await?;

        // Verify connection works
        let ls_cmd = LsCommand::new().build()?;
        client.execute(&ls_cmd).await?;

        // Store in pool
        let managed = ManagedConnection {
            client,
            pid,
            last_used: Instant::now(),
            is_healthy: true,
        };

        self.connections.insert(pid, managed);
        self.status.insert(pid, KittyConnectionStatus::Ready);

        Ok(self.connections.get_mut(&pid).map(|c| &mut c.client).unwrap())
    }
}
```

### 2. Retry Logic

**Exponential Backoff:**

```rust
pub async fn execute_with_retry(
    &mut self,
    pid: i32,
    command: &KittyMessage,
) -> Result<KittyResponse, KittyError> {
    let mut retry_count = 0;
    let max_retries = self.config.max_retries;
    let base_delay = Duration::from_millis(self.config.base_delay_ms);

    loop {
        match self.get_connection(pid).await {
            Ok(client) => {
                match client.execute(command).await {
                    Ok(response) => return Ok(response),
                    Err(KittyError::Connection(_)) => {
                        retry_count += 1;
                        if retry_count > max_retries {
                            return Err(KittyError::Connection(
                                ConnectionError::MaxRetriesExceeded(max_retries),
                            ));
                        }

                        // Remove failed connection
                        self.connections.remove(&pid);
                        self.status.insert(
                            pid,
                            KittyConnectionStatus::Failed(format!("Connection lost on attempt {}", retry_count)),
                        );

                        // Exponential backoff
                        let delay = base_delay * 2_u32.pow(retry_count);
                        tokio::time::sleep(delay).await;
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 3. Status Tracking

**Periodic Health Checks:**

```rust
impl KittyRegistry {
    // Background task to check connection health
    pub async fn start_health_checks(&mut self) {
        let check_interval = Duration::from_secs(self.config.health_check_interval_secs);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(check_interval).await;
                // Implementation would check all registered connections
            }
        });
    }

    // Check specific kitty status
    async fn check_status(&mut self, pid: i32) {
        if !self.status.contains_key(&pid) {
            let socket_path = get_kitty_socket_path(pid);

            let status = match timeout(
                Duration::from_secs(1),
                UnixStream::connect(&socket_path),
            ).await {
                Ok(_) => {
                    // Socket exists, try a quick command
                    match self.establish_connection(pid).await {
                        Ok(_) => KittyConnectionStatus::Ready,
                        Err(e) => KittyConnectionStatus::Failed(e.to_string()),
                    }
                }
                Err(_) => KittyConnectionStatus::NoSocket,
            };

            self.status.insert(pid, status);
        }
    }
}
```

### 4. Corrected Focus Lost Behavior

**Problem:** Current implementation iterates through ALL kitty windows on focus lost.

**Solution:** Only affect the kitty that lost focus:

```rust
// Track which kitty currently has focus
struct FocusTracker {
    current_focused_kitty: Option<i32>,  // PID of kitty with focus
}

impl FocusTracker {
    // When focus is gained
    pub fn on_focus_gained(&mut self, pid: i32) {
        self.current_focused_kitty = Some(pid);
    }

    // When focus is lost
    pub fn on_focus_lost(&mut self) -> Option<i32> {
        self.current_focused_kitty.take()
    }
}

// In event loop:
niri_ipc::Event::WindowFocusChanged { id } => {
    match id {
        Some(focused_id) => {
            if let Some(window) = windows.get(&focused_id) {
                if is_kitty_window(&window.app_id, &args.app_id) {
                    if let Some(pid) = window.pid {
                        focus_tracker.on_focus_gained(pid);
                        // Only increase font size for THIS kitty
                        registry.execute_with_retry(pid, &font_increase_cmd).await?;
                    }
                }
            }
        }
        None => {
            // Focus lost - only affect the kitty that had focus
            if let Some(pid) = focus_tracker.on_focus_lost() {
                // Only decrease font size for THIS kitty
                registry.execute_with_retry(pid, &font_decrease_cmd).await?;
            }
        }
    }
}
```

### 5. Configuration

```rust
pub struct RegistryConfig {
    pub connect_timeout_secs: u64,
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_connections: usize,
    pub health_check_interval_secs: u64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 2,
            max_retries: 3,
            base_delay_ms: 100,
            max_connections: 10,
            health_check_interval_secs: 30,
        }
    }
}
```

### 6. Event Flow

```
+-------------------+         +---------------------+         +-------------------+
|  Kitty Opens    |         |  Focus Gained      |         |  Focus Lost      |
+-------------------+         +---------------------+         +-------------------+
         |                            |                            |
         v                            v                            v
+-------------------+         +---------------------+         +-------------------+
| Registry Status  |         | Registry Status    |         | Registry Status    |
| Check          |         | Check (optional)  |         | (already known)  |
+-------------------+         +---------------------+         +-------------------+
         |                            |                            |
         v                            v                            v
+-------------------+         +---------------------+         +-------------------+
| Connection Pool |         | Get Connection     |         | Execute Command   |
| (get/create)   |         | (reuse or new)    |         | with Retry        |
+-------------------+         +---------------------+         +-------------------+
         |                                                        |
         v                                                        v
+-------------------+         +---------------------+         +-------------------+
| Font Increase   |         | Send to Kitty     |         | Font Decrease    |
| x3             |         | (async, retry)    |         | x3               |
+-------------------+         +---------------------+         +-------------------+
```

### 7. Resource Management

**Connection Cleanup:**

```rust
impl KittyRegistry {
    // Clean up idle connections
    pub fn cleanup_idle(&mut self) {
        let idle_threshold = Duration::from_secs(300); // 5 minutes
        let now = Instant::now();

        self.connections.retain(|pid, conn| {
            if now.duration_since(conn.last_used) > idle_threshold {
                conn.client.close().await.ok();
                self.status.remove(pid);
                false // Remove from pool
            } else {
                true // Keep in pool
            }
        });
    }

    // Close all connections on shutdown
    pub async fn shutdown(&mut self) {
        for (pid, mut conn) in self.connections.drain() {
            conn.client.close().await.ok();
            self.status.remove(&pid);
        }
    }
}
```

## Integration with kitty-focus-tracker

### Main Function Changes

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Initialize registry with config
    let mut registry = KittyRegistry::new_with_config(
        RegistryConfig {
            connect_timeout_secs: args.socket_timeout,
            max_retries: args.max_retries,
            ..RegistryConfig::default()
        }
    );

    let mut focus_tracker = FocusTracker::new();
    let mut windows: HashMap<u64, KittyWindow> = HashMap::new();

    // Initialize niri IPC connection (blocking)
    let mut niri_socket = Socket::connect()?;

    // ... event loop with async registry operations
    loop {
        match read_event() {
            Ok(event) => match event {
                // Window opened - check kitty status in background
                niri_ipc::Event::WindowOpenedOrChanged { window } => {
                    if let Some(ref app_id) = window.app_id {
                        if is_kitty_window(app_id, &args.app_id) {
                            if let Some(pid) = window.pid {
                                // Async check without blocking event loop
                                tokio::spawn(async move {
                                    registry.check_status(pid).await;
                                });
                            }

                            windows.insert(window.id, KittyWindow {
                                app_id: app_id.clone(),
                                pid: window.pid,
                            });
                        }
                    }
                }

                // Focus gained - only affect this kitty
                niri_ipc::Event::WindowFocusChanged { id } => {
                    match id {
                        Some(focused_id) => {
                            if let Some(window) = windows.get(&focused_id) {
                                if is_kitty_window(&window.app_id, &args.app_id) {
                                    if let Some(pid) = window.pid {
                                        focus_tracker.on_focus_gained(pid);

                                        let mut font_increase_cmd = SetFontSizeCommand::new(0)
                                            .increment_op("+")
                                            .build()?;

                                        // Send 3 times
                                        for _ in 0..3 {
                                            match registry.execute_with_retry(pid, &font_increase_cmd).await {
                                                Ok(_) => {
                                                    // Success - report status
                                                let event = FocusEvent::FocusGained {
                                                    window_id: focused_id,
                                                    app_id: window.app_id.clone(),
                                                    zooming: Some(ZoomingStatus::Success {
                                                        pid,
                                                        font_adjustment: "+3",
                                                        message: "Font size increased by 3 increments".to_string(),
                                                    }),
                                                };
                                                    println!("{}", serde_json::to_string(&event).unwrap());
                                                }
                                                Err(e) => {
                                                    // Failure - report status
                                                    let event = FocusEvent::FocusGained {
                                                        window_id: focused_id,
                                                        app_id: window.app_id.clone(),
                                                        zooming: Some(ZoomingStatus::Failed {
                                                            pid,
                                                            error: e.to_string(),
                                                        }),
                                                    };
                                                    println!("{}", serde_json::to_string(&event).unwrap());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Focus lost - only affect the kitty that lost focus
                        None => {
                            if let Some(pid) = focus_tracker.on_focus_lost() {
                                let mut font_decrease_cmd = SetFontSizeCommand::new(0)
                                    .increment_op("-")
                                    .build()?;

                                for _ in 0..3 {
                                    match registry.execute_with_retry(pid, &font_decrease_cmd).await {
                                        Ok(_) => {
                                            let event = FocusEvent::FocusLost {
                                                zooming: Some(ZoomingStatus::Success {
                                                    pid,
                                                    font_adjustment: "-3",
                                                    message: "Font size decreased by 3 increments".to_string(),
                                                }),
                                            };
                                            println!("{}", serde_json::to_string(&event).unwrap());
                                        }
                                        Err(e) => {
                                            let event = FocusEvent::FocusLost {
                                                zooming: Some(ZoomingStatus::Failed {
                                                    pid,
                                                    error: e.to_string(),
                                                }),
                                            };
                                            println!("{}", serde_json::to_string(&event).unwrap());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // ... other events
            }
            Err(e) => {
                eprintln!("Error reading event: {:?}", e);
                break;
            }
        }
    }

    // Cleanup
    registry.shutdown().await;
    Ok(())
}
```

## Testing Scenarios

1. **Connection pooling reuse:**
   - First focus: establish new connection
   - Subsequent focus on same kitty: reuse connection
   - Verify last_used timestamp updates

2. **Retry on transient failure:**
   - Simulate socket disconnect
   - Verify retry with exponential backoff
   - Verify max_retries limit

3. **Focus tracking correctness:**
   - Focus kitty A, then kitty B
   - Lose focus from kitty B
   - Verify only kitty B font size decreases

4. **Health check cleanup:**
   - Leave kitty idle > 5 minutes
   - Verify connection closed and removed from pool

5. **Multiple kitty instances:**
   - Launch 3 kitty windows (mix of configured/unconfigured)
   - Focus each sequentially
   - Verify only focused kitty affected

## Open Questions

1. Should we do background status checks proactively or lazily on first use?
   - Lazy: Simpler, first call is slower
   - Proactive: Better UX, catches failures earlier

2. What's the optimal health check interval?
   - Too frequent: High overhead
   - Too infrequent: Stale connections

3. Should we report connection pool statistics?
   - Total connections
   - Active vs idle
   - Reuse rate

## Success Criteria

- [ ] Connection pooling implemented
- [ ] Retry logic with exponential backoff
- [ ] Focus lost only affects single kitty
- [ ] Health check and cleanup of idle connections
- [ ] Status tracking per kitty PID
- [ ] Graceful shutdown handling
- [ ] Comprehensive error reporting
- [ ] Works with mixed kitty configurations
