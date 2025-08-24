import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { blockchainApi, Transaction } from '@/lib/blockchain-api';
import { Clock, ArrowRight, Hash, Copy, CheckCircle, Wallet } from 'lucide-react';
import { format } from 'date-fns';
import { useState } from 'react';

export function PendingTransactions() {
  const [copiedHash, setCopiedHash] = useState<string | null>(null);

  const { data: transactions, isLoading, error } = useQuery<Transaction[]>({
    queryKey: ['pending-transactions'],
    queryFn: blockchainApi.getPendingTransactions,
    refetchInterval: 5000, // Refetch every 5 seconds
  });

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedHash(text);
      setTimeout(() => setCopiedHash(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const formatHash = (hash: string) => {
    return `${hash.substring(0, 8)}...${hash.substring(hash.length - 8)}`;
  };

  const formatAddress = (address: string) => {
    return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
  };

  if (isLoading) {
    return (
      <Card className="bg-gradient-card border-border/50">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-accent" />
            Pending Transactions
          </CardTitle>
          <CardDescription>Transactions awaiting confirmation</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {[...Array(3)].map((_, i) => (
              <div key={i} className="p-3 bg-muted/20 rounded-lg animate-pulse">
                <div className="h-4 w-24 bg-muted rounded mb-2" />
                <div className="h-3 w-48 bg-muted rounded mb-1" />
                <div className="h-3 w-32 bg-muted rounded" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="bg-gradient-card border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Error Loading Transactions
          </CardTitle>
        <CardDescription>
          Unable to fetch pending transactions.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground">Unable to fetch pending transactions.</p>
      </CardContent>
      </Card>
    );
  }

  return (
    <Card className="bg-gradient-card border-border/50">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Clock className="h-5 w-5 text-accent animate-pulse-glow" />
          Pending Transactions
          {transactions && transactions.length > 0 && (
            <Badge variant="secondary" className="ml-2">
              {transactions.length}
            </Badge>
          )}
        </CardTitle>
        <CardDescription>
          Transactions waiting to be included in the next block
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-80">
          {transactions && transactions.length > 0 ? (
            <div className="space-y-3">
              {transactions.map((tx) => (
                <div
                  key={tx.id}
                  className="p-4 bg-muted/10 rounded-lg border border-border/50 hover:border-accent/50 transition-smooth"
                >
                  {/* Transaction Header */}
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                      <Hash className="h-4 w-4 text-accent" />
                      <code className="text-sm font-mono text-accent">
                        {formatHash(tx.hash)}
                      </code>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => copyToClipboard(tx.hash)}
                        className="h-6 w-6 p-0"
                      >
                        {copiedHash === tx.hash ? (
                          <CheckCircle className="h-3 w-3 text-secondary" />
                        ) : (
                          <Copy className="h-3 w-3" />
                        )}
                      </Button>
                    </div>
                    <Badge variant="outline" className="text-accent border-accent/50">
                      {tx.amount} LDB
                    </Badge>
                  </div>

                  {/* Transaction Details */}
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                      <div className="flex items-center gap-2">
                        <Wallet className="h-3 w-3 text-muted-foreground" />
                        <span className="text-muted-foreground">From:</span>
                        <code className="text-xs font-mono bg-background/50 px-2 py-1 rounded">
                          {formatAddress(tx.from)}
                        </code>
                      </div>
                      <ArrowRight className="h-4 w-4 text-muted-foreground" />
                      <div className="flex items-center gap-2">
                        <span className="text-muted-foreground">To:</span>
                        <code className="text-xs font-mono bg-background/50 px-2 py-1 rounded">
                          {formatAddress(tx.to)}
                        </code>
                      </div>
                    </div>

                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                      <span>
                        Created: {format(new Date(tx.timestamp * 1000), 'MMM dd, HH:mm:ss')}
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        Pending
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12 text-muted-foreground">
              <Clock className="h-16 w-16 mx-auto mb-4 opacity-50" />
              <h3 className="font-medium mb-2">No Pending Transactions</h3>
              <p className="text-sm">
                Create a transaction to see it appear here.
              </p>
              <p className="text-xs mt-2">
                Transactions will wait here until the next block is mined.
              </p>
            </div>
          )}
        </ScrollArea>

        {transactions && transactions.length > 0 && (
          <div className="mt-4 p-3 bg-accent/5 border border-accent/20 rounded-lg">
            <div className="flex items-center gap-2 text-sm text-accent">
              <Clock className="h-4 w-4" />
              <span className="font-medium">
                {transactions.length} transaction(s) ready for mining
              </span>
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              Mine a new block to confirm these transactions and add them to the blockchain.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}