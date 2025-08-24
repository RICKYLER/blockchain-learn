import { useState, useEffect } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { BlockchainWebSocket } from '@/lib/blockchain-api';
import { Wifi, WifiOff, RefreshCw, Zap } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export function WebSocketStatus() {
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected'>('disconnected');
  const [lastMessage, setLastMessage] = useState<any>(null);
  const [ws, setWs] = useState<BlockchainWebSocket | null>(null);
  const { toast } = useToast();

  useEffect(() => {
    const websocket = new BlockchainWebSocket(
      (data) => {
        setLastMessage({
          ...data,
          timestamp: Date.now()
        });
        
        // Show notification for important events
        if (data.type === 'block_mined') {
          toast({
            title: "New Block Mined! ⛏️",
            description: `Block #${data.block?.index} has been added to the blockchain.`,
            duration: 4000,
          });
        } else if (data.type === 'transaction_created') {
          toast({
            title: "New Transaction 💰",
            description: "A new transaction has been added to the pending pool.",
            duration: 3000,
          });
        }
      },
      (error) => {
        console.error('WebSocket error:', error);
        setConnectionStatus('disconnected');
      },
      () => {
        setConnectionStatus('connected');
        toast({
          title: "Connected to LedgerDB",
          description: "Real-time updates are now active.",
          duration: 3000,
        });
      },
      () => {
        setConnectionStatus('disconnected');
      }
    );

    setWs(websocket);
    setConnectionStatus('connecting');
    websocket.connect();

    return () => {
      websocket.disconnect();
    };
  }, [toast]);

  const handleReconnect = () => {
    if (ws) {
      setConnectionStatus('connecting');
      ws.connect();
    }
  };

  const getStatusColor = () => {
    switch (connectionStatus) {
      case 'connected': return 'text-secondary';
      case 'connecting': return 'text-accent';
      case 'disconnected': return 'text-destructive';
      default: return 'text-muted-foreground';
    }
  };

  const getStatusIcon = () => {
    switch (connectionStatus) {
      case 'connected': return <Wifi className="h-4 w-4" />;
      case 'connecting': return <RefreshCw className="h-4 w-4 animate-spin" />;
      case 'disconnected': return <WifiOff className="h-4 w-4" />;
      default: return <WifiOff className="h-4 w-4" />;
    }
  };

  return (
    <Card className="bg-gradient-card border-border/50">
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`${getStatusColor()}`}>
              {getStatusIcon()}
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">WebSocket Status</span>
                <Badge 
                  variant={connectionStatus === 'connected' ? 'default' : 'secondary'}
                  className={
                    connectionStatus === 'connected' 
                      ? 'bg-secondary text-secondary-foreground' 
                      : connectionStatus === 'connecting'
                      ? 'bg-accent text-accent-foreground animate-pulse'
                      : 'bg-destructive text-destructive-foreground'
                  }
                >
                  {connectionStatus === 'connected' && 'Connected'}
                  {connectionStatus === 'connecting' && 'Connecting...'}
                  {connectionStatus === 'disconnected' && 'Disconnected'}
                </Badge>
              </div>
              {lastMessage && (
                <p className="text-xs text-muted-foreground">
                  Last update: {new Date(lastMessage.timestamp).toLocaleTimeString()}
                </p>
              )}
            </div>
          </div>

          {connectionStatus === 'disconnected' && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleReconnect}
              className="gap-2"
            >
              <RefreshCw className="h-3 w-3" />
              Reconnect
            </Button>
          )}

          {connectionStatus === 'connected' && lastMessage && (
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Zap className="h-3 w-3 text-secondary" />
              <span>Live updates active</span>
            </div>
          )}
        </div>

        {lastMessage && (
          <div className="mt-3 p-2 bg-muted/20 rounded text-xs">
            <div className="flex items-center gap-2 mb-1">
              <span className="font-medium">Latest Event:</span>
              <Badge variant="outline" className="text-xs">
                {lastMessage.type?.replace('_', ' ').toUpperCase() || 'UNKNOWN'}
              </Badge>
            </div>
            <code className="text-xs text-muted-foreground">
              {JSON.stringify(lastMessage, null, 2).substring(0, 100)}...
            </code>
          </div>
        )}
      </CardContent>
    </Card>
  );
}