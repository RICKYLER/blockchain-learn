import axios from 'axios';

// Configure base URL for your Rust backend
const BASE_URL = 'http://localhost:3000';

const api = axios.create({
  baseURL: BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Types for blockchain data
export interface Block {
  id: string;
  index: number;
  timestamp: number;
  previous_hash: string;
  hash: string;
  merkle_root: string;
  nonce: number;
  difficulty: number;
  transactions: Transaction[];
}

export interface Transaction {
  id: string;
  from: string;
  to: string;
  amount: number;
  timestamp: number;
  signature?: string;
  hash: string;
}

export interface BlockchainInfo {
  chain_length: number;
  latest_block_hash: string;
  total_transactions: number;
  pending_transactions: number;
  difficulty: number;
  network_hash_rate?: number;
}

export interface MiningRequest {
  transactions?: string[];
  reward_address?: string;
}

export interface CreateTransactionRequest {
  from: string;
  to: string;
  amount: number;
}

// API functions
export const blockchainApi = {
  // Blockchain info
  getBlockchainInfo: (): Promise<BlockchainInfo> =>
    api.get('/api/stats').then(res => res.data),

  // Blocks
  getBlocks: (): Promise<Block[]> =>
    api.get('/api/blocks').then(res => res.data),

  getBlock: (id: string): Promise<Block> =>
    api.get(`/api/blocks/${id}`).then(res => res.data),

  // Mining
  mineBlock: (request?: MiningRequest): Promise<Block> =>
    api.post('/api/mine', request || {}).then(res => res.data),

  // Transactions
  createTransaction: (transaction: CreateTransactionRequest): Promise<Transaction> =>
    api.post('/api/submit_transaction', transaction).then(res => res.data),

  getTransaction: (id: string): Promise<Transaction> =>
    api.get(`/api/transactions/${id}`).then(res => res.data),

  getPendingTransactions: (): Promise<Transaction[]> =>
    api.get('/api/transactions').then(res => res.data),

  // Additional endpoints available in backend
  getBalance: (address: string): Promise<any> =>
    api.get(`/api/balance/${address}`).then(res => res.data),

  getHealth: (): Promise<any> =>
    api.get('/api/health').then(res => res.data),
};

// WebSocket connection for real-time updates
export class BlockchainWebSocket {
  private ws: WebSocket | null = null;
  private reconnectInterval: number = 5000;
  private maxReconnectAttempts: number = 5;
  private reconnectAttempts: number = 0;

  constructor(
    private onMessage: (data: any) => void,
    private onError: (error: Event) => void = () => {},
    private onConnect: () => void = () => {},
    private onDisconnect: () => void = () => {}
  ) {}

  connect(): void {
    try {
      const wsUrl = BASE_URL.replace('http', 'ws') + '/ws';
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        this.onConnect();
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.onMessage(data);
        } catch (error) {
          console.error('Error parsing WebSocket message:', error);
        }
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.onDisconnect();
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.onError(error);
      };
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.attemptReconnect();
    }
  }

  private attemptReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`Attempting to reconnect... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
      setTimeout(() => this.connect(), this.reconnectInterval);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  send(data: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    } else {
      console.warn('WebSocket is not connected');
    }
  }
}