import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { blockchainApi, Block } from '@/lib/blockchain-api';
import { 
  Blocks, 
  Hash, 
  Clock, 
  ArrowRight, 
  Cpu, 
  ChevronDown, 
  ChevronUp,
  Copy,
  CheckCircle
} from 'lucide-react';
import { format } from 'date-fns';

export function BlocksList() {
  const [expandedBlock, setExpandedBlock] = useState<string | null>(null);
  const [copiedHash, setCopiedHash] = useState<string | null>(null);

  const { data: blocks, isLoading, error } = useQuery<Block[]>({
    queryKey: ['blocks'],
    queryFn: blockchainApi.getBlocks,
    refetchInterval: 10000, // Refetch every 10 seconds
  });

  const copyToClipboard = async (text: string, type: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedHash(`${type}-${text}`);
      setTimeout(() => setCopiedHash(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const toggleExpanded = (blockId: string) => {
    setExpandedBlock(expandedBlock === blockId ? null : blockId);
  };

  const formatHash = (hash: string) => {
    return `${hash.substring(0, 8)}...${hash.substring(hash.length - 8)}`;
  };

  if (isLoading) {
    return (
      <Card className="bg-gradient-card border-border/50">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Blocks className="h-5 w-5 text-primary" />
            Recent Blocks
          </CardTitle>
          <CardDescription>Latest blocks in the blockchain</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="p-4 bg-muted/20 rounded-lg animate-pulse">
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
            <Blocks className="h-5 w-5" />
            Error Loading Blocks
          </CardTitle>
        <CardDescription>
          Unable to fetch blocks from the blockchain.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground">Unable to fetch blocks from the blockchain.</p>
      </CardContent>
      </Card>
    );
  }

  return (
    <Card className="bg-gradient-card border-border/50">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Blocks className="h-5 w-5 text-primary" />
          Recent Blocks
        </CardTitle>
        <CardDescription>
          Latest blocks in the LedgerDB blockchain
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-96">
          <div className="space-y-4">
            {blocks?.slice().reverse().map((block) => (
              <div
                key={block.id}
                className="p-4 bg-muted/10 rounded-lg border border-border/50 hover:border-primary/50 transition-smooth"
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <Badge variant="outline" className="text-primary">
                      Block #{block.index}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      {format(new Date(block.timestamp * 1000), 'MMM dd, HH:mm:ss')}
                    </span>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => toggleExpanded(block.id)}
                  >
                    {expandedBlock === block.id ? (
                      <ChevronUp className="h-4 w-4" />
                    ) : (
                      <ChevronDown className="h-4 w-4" />
                    )}
                  </Button>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <Hash className="h-4 w-4 text-primary" />
                    <span className="text-muted-foreground">Hash:</span>
                    <code className="bg-background/50 px-2 py-1 rounded text-primary font-mono">
                      {formatHash(block.hash)}
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard(block.hash, 'hash')}
                      className="h-6 w-6 p-0"
                    >
                      {copiedHash === `hash-${block.hash}` ? (
                        <CheckCircle className="h-3 w-3 text-secondary" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </Button>
                  </div>

                  <div className="flex items-center gap-4 text-sm text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <ArrowRight className="h-3 w-3" />
                      <span>{block.transactions.length} transactions</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Cpu className="h-3 w-3" />
                      <span>Nonce: {block.nonce.toLocaleString()}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      <span>Difficulty: {block.difficulty}</span>
                    </div>
                  </div>
                </div>

                {expandedBlock === block.id && (
                  <div className="mt-4 pt-4 border-t border-border/50 space-y-3">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                      <div>
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-muted-foreground">Previous Hash:</span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(block.previous_hash, 'prev')}
                            className="h-6 w-6 p-0"
                          >
                            {copiedHash === `prev-${block.previous_hash}` ? (
                              <CheckCircle className="h-3 w-3 text-secondary" />
                            ) : (
                              <Copy className="h-3 w-3" />
                            )}
                          </Button>
                        </div>
                        <code className="bg-background/50 px-2 py-1 rounded text-xs font-mono block">
                          {block.previous_hash}
                        </code>
                      </div>
                      <div>
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-muted-foreground">Merkle Root:</span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyToClipboard(block.merkle_root, 'merkle')}
                            className="h-6 w-6 p-0"
                          >
                            {copiedHash === `merkle-${block.merkle_root}` ? (
                              <CheckCircle className="h-3 w-3 text-secondary" />
                            ) : (
                              <Copy className="h-3 w-3" />
                            )}
                          </Button>
                        </div>
                        <code className="bg-background/50 px-2 py-1 rounded text-xs font-mono block">
                          {block.merkle_root}
                        </code>
                      </div>
                    </div>

                    {block.transactions.length > 0 && (
                      <div>
                        <h4 className="font-medium mb-2">Transactions in this block:</h4>
                        <div className="space-y-2 max-h-32 overflow-y-auto">
                          {block.transactions.map((tx) => (
                            <div key={tx.id} className="p-2 bg-background/30 rounded text-sm">
                              <div className="flex justify-between items-center">
                                <span className="font-mono text-xs">{formatHash(tx.hash)}</span>
                                <Badge variant="secondary" className="text-xs">
                                  {tx.amount} LDB
                                </Badge>
                              </div>
                              <div className="text-xs text-muted-foreground mt-1">
                                From: {formatHash(tx.from)} → To: {formatHash(tx.to)}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
            
            {(!blocks || blocks.length === 0) && (
              <div className="text-center py-8 text-muted-foreground">
                <Blocks className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No blocks found in the blockchain yet.</p>
                <p className="text-sm">Start mining to create the first block!</p>
              </div>
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}