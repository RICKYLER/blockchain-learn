import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/hooks/use-toast';
import { blockchainApi, MiningRequest } from '@/lib/blockchain-api';
import { Pickaxe, Zap, Clock, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';

export function MiningControls() {
  const [rewardAddress, setRewardAddress] = useState('');
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const miningMutation = useMutation({
    mutationFn: (request: MiningRequest) => blockchainApi.mineBlock(request),
    onSuccess: (block) => {
      toast({
        title: "Block Mined Successfully! ⛏️",
        description: `Block #${block.index} has been added to the blockchain.`,
        duration: 5000,
      });
      
      // Invalidate and refetch relevant queries
      queryClient.invalidateQueries({ queryKey: ['blockchain-info'] });
      queryClient.invalidateQueries({ queryKey: ['blocks'] });
      queryClient.invalidateQueries({ queryKey: ['pending-transactions'] });
      
      // Reset form
      setRewardAddress('');
    },
    onError: (error: any) => {
      console.error('Mining error:', error);
      toast({
        title: "Mining Failed",
        description: error?.response?.data?.message || "An error occurred while mining the block.",
        variant: "destructive",
        duration: 5000,
      });
    },
  });

  const handleMineBlock = () => {
    const request: MiningRequest = {};
    
    if (rewardAddress.trim()) {
      request.reward_address = rewardAddress.trim();
    }

    miningMutation.mutate(request);
  };

  const generateRandomAddress = () => {
    // Generate a simple random address for demo purposes
    const chars = '0123456789abcdef';
    let address = '0x';
    for (let i = 0; i < 40; i++) {
      address += chars[Math.floor(Math.random() * chars.length)];
    }
    setRewardAddress(address);
  };

  return (
    <Card className="bg-gradient-card border-border/50">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Pickaxe className="h-5 w-5 text-accent" />
          Mining Controls
        </CardTitle>
        <CardDescription>
          Mine new blocks and earn rewards on the LedgerDB blockchain
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Mining Status */}
        <div className="p-4 bg-muted/10 rounded-lg border border-border/50">
          <div className="flex items-center justify-between mb-3">
            <h3 className="font-medium">Mining Status</h3>
            <Badge 
              variant={miningMutation.isPending ? "default" : "secondary"}
              className={miningMutation.isPending ? "animate-pulse-glow" : ""}
            >
              {miningMutation.isPending ? (
                <>
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                  Mining...
                </>
              ) : (
                <>
                  <CheckCircle className="h-3 w-3 mr-1" />
                  Ready
                </>
              )}
            </Badge>
          </div>
          
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 text-sm">
            <div className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-accent" />
              <span className="text-muted-foreground">Algorithm:</span>
              <span className="font-medium">Proof of Work</span>
            </div>
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-primary" />
              <span className="text-muted-foreground">Est. Time:</span>
              <span className="font-medium">~30-60s</span>
            </div>
            <div className="flex items-center gap-2">
              <Pickaxe className="h-4 w-4 text-secondary" />
              <span className="text-muted-foreground">Reward:</span>
              <span className="font-medium">50 LDB</span>
            </div>
          </div>
        </div>

        <Separator />

        {/* Reward Address Input */}
        <div className="space-y-3">
          <Label htmlFor="reward-address" className="text-sm font-medium">
            Reward Address (Optional)
          </Label>
          <div className="flex gap-2">
            <Input
              id="reward-address"
              type="text"
              placeholder="0x... (leave empty for default address)"
              value={rewardAddress}
              onChange={(e) => setRewardAddress(e.target.value)}
              className="font-mono text-sm"
              disabled={miningMutation.isPending}
            />
            <Button
              type="button"
              variant="outline"
              onClick={generateRandomAddress}
              disabled={miningMutation.isPending}
              className="whitespace-nowrap"
            >
              Generate
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            The address where mining rewards will be sent. If not specified, rewards go to the default miner address.
          </p>
        </div>

        <Separator />

        {/* Mining Button */}
        <div className="space-y-4">
          <Button
            onClick={handleMineBlock}
            disabled={miningMutation.isPending}
            variant="mining"
            size="lg"
            className="w-full"
          >
            {miningMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                Mining Block...
              </>
            ) : (
              <>
                <Pickaxe className="h-4 w-4 mr-2" />
                Start Mining
              </>
            )}
          </Button>

          {miningMutation.isPending && (
            <div className="p-3 bg-accent/10 border border-accent/30 rounded-lg">
              <div className="flex items-center gap-2 text-sm text-accent">
                <Clock className="h-4 w-4" />
                <span className="font-medium">Mining in progress...</span>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Searching for valid nonce and computing proof-of-work. This may take several seconds.
              </p>
            </div>
          )}

          {miningMutation.isError && (
            <div className="p-3 bg-destructive/10 border border-destructive/30 rounded-lg">
              <div className="flex items-center gap-2 text-sm text-destructive">
                <AlertCircle className="h-4 w-4" />
                <span className="font-medium">Mining failed</span>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                Please check your connection to the LedgerDB backend and try again.
              </p>
            </div>
          )}
        </div>

        {/* Mining Info */}
        <div className="p-3 bg-primary/5 border border-primary/20 rounded-lg">
          <h4 className="text-sm font-medium text-primary mb-2">Mining Information</h4>
          <ul className="text-xs text-muted-foreground space-y-1">
            <li>• Mining validates pending transactions and secures the network</li>
            <li>• Successful mining adds a new block and earns rewards</li>
            <li>• Difficulty adjusts automatically based on network hashrate</li>
            <li>• Each block includes a Merkle tree for transaction verification</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
}