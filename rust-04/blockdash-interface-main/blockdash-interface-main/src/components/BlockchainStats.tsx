import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { blockchainApi, BlockchainInfo } from '@/lib/blockchain-api';
import { Activity, Blocks, Hash, Clock, Zap, TrendingUp } from 'lucide-react';

export function BlockchainStats() {
  const { data: info, isLoading, error } = useQuery<BlockchainInfo>({
    queryKey: ['blockchain-info'],
    queryFn: blockchainApi.getBlockchainInfo,
    refetchInterval: 5000, // Refetch every 5 seconds
  });

  if (isLoading) {
    return (
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {[...Array(4)].map((_, i) => (
          <Card key={i} className="bg-gradient-card border-border/50">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="h-4 w-20 bg-muted animate-pulse rounded" />
              <div className="h-4 w-4 bg-muted animate-pulse rounded" />
            </CardHeader>
            <CardContent>
              <div className="h-8 w-24 bg-muted animate-pulse rounded mb-1" />
              <div className="h-3 w-32 bg-muted animate-pulse rounded" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <Card className="bg-gradient-card border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive">Connection Error</CardTitle>
          <CardDescription>
            Unable to connect to LedgerDB backend. Please ensure the Rust server is running on localhost:3000.
          </CardDescription>
        </CardHeader>
      </Card>
    );
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card className="bg-gradient-card border-border/50 hover:shadow-primary transition-smooth">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Chain Length
          </CardTitle>
          <Blocks className="h-4 w-4 text-primary" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-primary">
            {info?.chain_length?.toLocaleString() || '0'}
          </div>
          <p className="text-xs text-muted-foreground">
            Total blocks in chain
          </p>
        </CardContent>
      </Card>

      <Card className="bg-gradient-card border-border/50 hover:shadow-secondary transition-smooth">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Total Transactions
          </CardTitle>
          <Activity className="h-4 w-4 text-secondary" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-secondary">
            {info?.total_transactions?.toLocaleString() || '0'}
          </div>
          <p className="text-xs text-muted-foreground">
            Confirmed transactions
          </p>
        </CardContent>
      </Card>

      <Card className="bg-gradient-card border-border/50 hover:shadow-glow transition-smooth">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Pending Transactions
          </CardTitle>
          <Clock className="h-4 w-4 text-accent" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-accent">
            {info?.pending_transactions?.toLocaleString() || '0'}
          </div>
          <p className="text-xs text-muted-foreground">
            Awaiting confirmation
          </p>
          {info?.pending_transactions && info.pending_transactions > 0 && (
            <Badge variant="secondary" className="mt-1">
              Ready to mine
            </Badge>
          )}
        </CardContent>
      </Card>

      <Card className="bg-gradient-card border-border/50 hover:shadow-card transition-smooth">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Mining Difficulty
          </CardTitle>
          <Hash className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {info?.difficulty || '0'}
          </div>
          <p className="text-xs text-muted-foreground">
            Current difficulty level
          </p>
          <div className="flex items-center mt-2">
            <Badge variant="outline" className="text-xs">
              PoW Algorithm
            </Badge>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}