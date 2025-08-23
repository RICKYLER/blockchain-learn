//! Network utilities for the LedgerDB blockchain.
//!
//! This module provides network-related functionality including
//! peer management, network discovery, and communication utilities.

use crate::error::LedgerError;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub max_message_size: usize,
    pub default_port: u16,
    pub bootstrap_nodes: Vec<SocketAddr>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(60),
            max_message_size: 1024 * 1024, // 1MB
            default_port: 8333,
            bootstrap_nodes: vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333),
            ],
        }
    }
}

/// Peer information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub version: String,
    pub user_agent: String,
    pub services: u64,
    pub height: u64,
    pub last_seen: Instant,
    pub connection_time: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_outbound: bool,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(address: SocketAddr, is_outbound: bool) -> Self {
        let now = Instant::now();
        Self {
            address,
            version: "1.0.0".to_string(),
            user_agent: "LedgerDB/1.0.0".to_string(),
            services: 1, // NODE_NETWORK
            height: 0,
            last_seen: now,
            connection_time: now,
            bytes_sent: 0,
            bytes_received: 0,
            is_outbound,
        }
    }
    
    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
    
    /// Add sent bytes
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }
    
    /// Add received bytes
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }
    
    /// Get connection duration
    pub fn connection_duration(&self) -> Duration {
        Instant::now().duration_since(self.connection_time)
    }
    
    /// Check if peer is stale
    pub fn is_stale(&self, timeout: Duration) -> bool {
        Instant::now().duration_since(self.last_seen) > timeout
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub total_peers: usize,
    pub outbound_peers: usize,
    pub inbound_peers: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connection_attempts: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
}

impl NetworkStats {
    /// Calculate connection success rate
    pub fn connection_success_rate(&self) -> f64 {
        if self.connection_attempts == 0 {
            0.0
        } else {
            self.successful_connections as f64 / self.connection_attempts as f64
        }
    }
    
    /// Calculate average bytes per message sent
    pub fn avg_bytes_per_message_sent(&self) -> f64 {
        if self.messages_sent == 0 {
            0.0
        } else {
            self.total_bytes_sent as f64 / self.messages_sent as f64
        }
    }
    
    /// Calculate average bytes per message received
    pub fn avg_bytes_per_message_received(&self) -> f64 {
        if self.messages_received == 0 {
            0.0
        } else {
            self.total_bytes_received as f64 / self.messages_received as f64
        }
    }
}

/// Peer manager for handling network connections
#[derive(Debug)]
pub struct PeerManager {
    peers: HashMap<SocketAddr, PeerInfo>,
    config: NetworkConfig,
    stats: NetworkStats,
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            peers: HashMap::new(),
            config,
            stats: NetworkStats::default(),
        }
    }
    
    /// Add a new peer
    pub fn add_peer(&mut self, address: SocketAddr, is_outbound: bool) -> Result<(), LedgerError> {
        if self.peers.len() >= self.config.max_peers {
            return Err(LedgerError::Network("Maximum peers reached".to_string()));
        }
        
        if self.peers.contains_key(&address) {
            return Err(LedgerError::Network("Peer already exists".to_string()));
        }
        
        let peer = PeerInfo::new(address, is_outbound);
        self.peers.insert(address, peer);
        
        self.update_stats();
        
        Ok(())
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, address: &SocketAddr) -> Option<PeerInfo> {
        let peer = self.peers.remove(address);
        self.update_stats();
        peer
    }
    
    /// Get peer information
    pub fn get_peer(&self, address: &SocketAddr) -> Option<&PeerInfo> {
        self.peers.get(address)
    }
    
    /// Get mutable peer information
    pub fn get_peer_mut(&mut self, address: &SocketAddr) -> Option<&mut PeerInfo> {
        self.peers.get_mut(address)
    }
    
    /// Get all peers
    pub fn get_all_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }
    
    /// Get outbound peers
    pub fn get_outbound_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().filter(|p| p.is_outbound).collect()
    }
    
    /// Get inbound peers
    pub fn get_inbound_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().filter(|p| !p.is_outbound).collect()
    }
    
    /// Remove stale peers
    pub fn remove_stale_peers(&mut self) -> Vec<SocketAddr> {
        let timeout = self.config.heartbeat_interval * 3; // 3x heartbeat interval
        let stale_addresses: Vec<SocketAddr> = self
            .peers
            .iter()
            .filter(|(_, peer)| peer.is_stale(timeout))
            .map(|(addr, _)| *addr)
            .collect();
        
        for addr in &stale_addresses {
            self.peers.remove(addr);
        }
        
        if !stale_addresses.is_empty() {
            self.update_stats();
        }
        
        stale_addresses
    }
    
    /// Update network statistics
    fn update_stats(&mut self) {
        self.stats.total_peers = self.peers.len();
        self.stats.outbound_peers = self.peers.values().filter(|p| p.is_outbound).count();
        self.stats.inbound_peers = self.peers.values().filter(|p| !p.is_outbound).count();
        
        self.stats.total_bytes_sent = self.peers.values().map(|p| p.bytes_sent).sum();
        self.stats.total_bytes_received = self.peers.values().map(|p| p.bytes_received).sum();
    }
    
    /// Get network statistics
    pub fn get_stats(&self) -> &NetworkStats {
        &self.stats
    }
    
    /// Get network configuration
    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }
    
    /// Check if we can accept more peers
    pub fn can_accept_peers(&self) -> bool {
        self.peers.len() < self.config.max_peers
    }
    
    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
    
    /// Record connection attempt
    pub fn record_connection_attempt(&mut self) {
        self.stats.connection_attempts += 1;
    }
    
    /// Record successful connection
    pub fn record_successful_connection(&mut self) {
        self.stats.successful_connections += 1;
    }
    
    /// Record failed connection
    pub fn record_failed_connection(&mut self) {
        self.stats.failed_connections += 1;
    }
    
    /// Record message sent
    pub fn record_message_sent(&mut self, bytes: u64) {
        self.stats.messages_sent += 1;
        self.stats.total_bytes_sent += bytes;
    }
    
    /// Record message received
    pub fn record_message_received(&mut self, bytes: u64) {
        self.stats.messages_received += 1;
        self.stats.total_bytes_received += bytes;
    }
}

/// Network message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Version,
    VerAck,
    Ping,
    Pong,
    GetAddr,
    Addr,
    Inv,
    GetData,
    Block,
    Transaction,
    GetBlocks,
    GetHeaders,
    Headers,
    NotFound,
    Reject,
}

impl MessageType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::Version => "version",
            MessageType::VerAck => "verack",
            MessageType::Ping => "ping",
            MessageType::Pong => "pong",
            MessageType::GetAddr => "getaddr",
            MessageType::Addr => "addr",
            MessageType::Inv => "inv",
            MessageType::GetData => "getdata",
            MessageType::Block => "block",
            MessageType::Transaction => "tx",
            MessageType::GetBlocks => "getblocks",
            MessageType::GetHeaders => "getheaders",
            MessageType::Headers => "headers",
            MessageType::NotFound => "notfound",
            MessageType::Reject => "reject",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "version" => Some(MessageType::Version),
            "verack" => Some(MessageType::VerAck),
            "ping" => Some(MessageType::Ping),
            "pong" => Some(MessageType::Pong),
            "getaddr" => Some(MessageType::GetAddr),
            "addr" => Some(MessageType::Addr),
            "inv" => Some(MessageType::Inv),
            "getdata" => Some(MessageType::GetData),
            "block" => Some(MessageType::Block),
            "tx" => Some(MessageType::Transaction),
            "getblocks" => Some(MessageType::GetBlocks),
            "getheaders" => Some(MessageType::GetHeaders),
            "headers" => Some(MessageType::Headers),
            "notfound" => Some(MessageType::NotFound),
            "reject" => Some(MessageType::Reject),
            _ => None,
        }
    }
}

/// Network message
#[derive(Debug, Clone)]
pub struct NetworkMessage {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: Instant,
}

impl NetworkMessage {
    /// Create new network message
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            payload,
            timestamp: Instant::now(),
        }
    }
    
    /// Get message size
    pub fn size(&self) -> usize {
        self.payload.len()
    }
    
    /// Get message age
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }
}

/// Network utilities
pub struct NetworkUtils;

impl NetworkUtils {
    /// Check if an IP address is local
    pub fn is_local_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_loopback()
                    || ipv4.is_private()
                    || ipv4.is_link_local()
                    || ipv4.is_broadcast()
                    || ipv4.is_multicast()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ipv6.is_multicast()
            }
        }
    }
    
    /// Check if an IP address is routable
    pub fn is_routable_ip(ip: &IpAddr) -> bool {
        !Self::is_local_ip(ip)
    }
    
    /// Parse socket address from string
    pub fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr, LedgerError> {
        addr_str.parse().map_err(|e| {
            LedgerError::Network(format!("Invalid socket address '{}': {}", addr_str, e))
        })
    }
    
    /// Format socket address as string
    pub fn format_socket_addr(addr: &SocketAddr) -> String {
        addr.to_string()
    }
    
    /// Get default bootstrap nodes
    pub fn get_default_bootstrap_nodes() -> Vec<SocketAddr> {
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333),
            // Add more bootstrap nodes as needed
        ]
    }
    
    /// Calculate network hash rate (placeholder)
    pub fn calculate_network_hashrate(
        difficulty: u32,
        block_time: Duration,
    ) -> f64 {
        // Simplified calculation
        // Real implementation would use actual difficulty and target block time
        let target_block_time_secs = block_time.as_secs_f64();
        if target_block_time_secs > 0.0 {
            difficulty as f64 / target_block_time_secs
        } else {
            0.0
        }
    }
    
    /// Validate network message size
    pub fn validate_message_size(size: usize, max_size: usize) -> Result<(), LedgerError> {
        if size > max_size {
            return Err(LedgerError::Network(format!(
                "Message size {} exceeds maximum {}",
                size, max_size
            )));
        }
        Ok(())
    }
    
    /// Generate random peer ID
    pub fn generate_peer_id() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Handshaking,
    Ready,
    Disconnecting,
}

/// Connection manager
#[derive(Debug)]
pub struct ConnectionManager {
    connections: HashMap<SocketAddr, ConnectionState>,
    max_connections: usize,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: HashMap::new(),
            max_connections,
        }
    }
    
    /// Add connection
    pub fn add_connection(&mut self, addr: SocketAddr) -> Result<(), LedgerError> {
        if self.connections.len() >= self.max_connections {
            return Err(LedgerError::Network("Maximum connections reached".to_string()));
        }
        
        self.connections.insert(addr, ConnectionState::Connecting);
        Ok(())
    }
    
    /// Update connection state
    pub fn update_connection_state(&mut self, addr: &SocketAddr, state: ConnectionState) {
        self.connections.insert(*addr, state);
    }
    
    /// Remove connection
    pub fn remove_connection(&mut self, addr: &SocketAddr) -> Option<ConnectionState> {
        self.connections.remove(addr)
    }
    
    /// Get connection state
    pub fn get_connection_state(&self, addr: &SocketAddr) -> Option<&ConnectionState> {
        self.connections.get(addr)
    }
    
    /// Get all connections
    pub fn get_all_connections(&self) -> &HashMap<SocketAddr, ConnectionState> {
        &self.connections
    }
    
    /// Get ready connections
    pub fn get_ready_connections(&self) -> Vec<SocketAddr> {
        self.connections
            .iter()
            .filter(|(_, state)| **state == ConnectionState::Ready)
            .map(|(addr, _)| *addr)
            .collect()
    }
    
    /// Connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    
    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.max_peers, 50);
        assert_eq!(config.default_port, 8333);
        assert!(!config.bootstrap_nodes.is_empty());
    }
    
    #[test]
    fn test_peer_info() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);
        let mut peer = PeerInfo::new(addr, true);
        
        assert_eq!(peer.address, addr);
        assert!(peer.is_outbound);
        assert_eq!(peer.bytes_sent, 0);
        assert_eq!(peer.bytes_received, 0);
        
        peer.add_bytes_sent(100);
        peer.add_bytes_received(200);
        
        assert_eq!(peer.bytes_sent, 100);
        assert_eq!(peer.bytes_received, 200);
    }
    
    #[test]
    fn test_peer_manager() {
        let config = NetworkConfig::default();
        let mut manager = PeerManager::new(config);
        
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);
        
        assert!(manager.add_peer(addr, true).is_ok());
        assert_eq!(manager.peer_count(), 1);
        
        // Try to add same peer again
        assert!(manager.add_peer(addr, true).is_err());
        
        // Remove peer
        assert!(manager.remove_peer(&addr).is_some());
        assert_eq!(manager.peer_count(), 0);
    }
    
    #[test]
    fn test_message_type() {
        assert_eq!(MessageType::Version.as_str(), "version");
        assert_eq!(MessageType::from_str("version"), Some(MessageType::Version));
        assert_eq!(MessageType::from_str("invalid"), None);
    }
    
    #[test]
    fn test_network_message() {
        let msg = NetworkMessage::new(MessageType::Ping, vec![1, 2, 3, 4]);
        assert_eq!(msg.message_type, MessageType::Ping);
        assert_eq!(msg.size(), 4);
    }
    
    #[test]
    fn test_network_utils() {
        let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let public_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
        
        assert!(NetworkUtils::is_local_ip(&local_ip));
        assert!(!NetworkUtils::is_local_ip(&public_ip));
        assert!(NetworkUtils::is_routable_ip(&public_ip));
        assert!(!NetworkUtils::is_routable_ip(&local_ip));
    }
    
    #[test]
    fn test_connection_manager() {
        let mut manager = ConnectionManager::new(2);
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8333);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8334);
        let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8335);
        
        assert!(manager.add_connection(addr1).is_ok());
        assert!(manager.add_connection(addr2).is_ok());
        
        // Should fail due to max connections
        assert!(manager.add_connection(addr3).is_err());
        
        manager.update_connection_state(&addr1, ConnectionState::Ready);
        assert_eq!(
            manager.get_connection_state(&addr1),
            Some(&ConnectionState::Ready)
        );
        
        assert_eq!(manager.get_ready_connections().len(), 1);
    }
}