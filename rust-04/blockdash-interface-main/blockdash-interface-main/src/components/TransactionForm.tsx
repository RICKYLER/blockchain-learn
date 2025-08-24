import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/hooks/use-toast';
import { blockchainApi, CreateTransactionRequest } from '@/lib/blockchain-api';
import { Send, Wallet, ArrowRight, CheckCircle, AlertCircle } from 'lucide-react';

export function TransactionForm() {
  const [fromAddress, setFromAddress] = useState('');
  const [toAddress, setToAddress] = useState('');
  const [amount, setAmount] = useState('');
  const { toast } = useToast();
  const queryClient = useQueryClient();

  const transactionMutation = useMutation({
    mutationFn: (transaction: CreateTransactionRequest) => 
      blockchainApi.createTransaction(transaction),
    onSuccess: (transaction) => {
      toast({
        title: "Transaction Created! 💰",
        description: `Transaction of ${transaction.amount} LDB has been added to the pending pool.`,
        duration: 5000,
      });
      
      // Invalidate and refetch relevant queries
      queryClient.invalidateQueries({ queryKey: ['blockchain-info'] });
      queryClient.invalidateQueries({ queryKey: ['pending-transactions'] });
      
      // Reset form
      setFromAddress('');
      setToAddress('');
      setAmount('');
    },
    onError: (error: any) => {
      console.error('Transaction error:', error);
      toast({
        title: "Transaction Failed",
        description: error?.response?.data?.message || "An error occurred while creating the transaction.",
        variant: "destructive",
        duration: 5000,
      });
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!fromAddress.trim() || !toAddress.trim() || !amount.trim()) {
      toast({
        title: "Invalid Input",
        description: "Please fill in all required fields.",
        variant: "destructive",
      });
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum <= 0) {
      toast({
        title: "Invalid Amount",
        description: "Please enter a valid positive amount.",
        variant: "destructive",
      });
      return;
    }

    const transaction: CreateTransactionRequest = {
      from: fromAddress.trim(),
      to: toAddress.trim(),
      amount: amountNum,
    };

    transactionMutation.mutate(transaction);
  };

  const generateRandomAddress = () => {
    const chars = '0123456789abcdef';
    let address = '0x';
    for (let i = 0; i < 40; i++) {
      address += chars[Math.floor(Math.random() * chars.length)];
    }
    return address;
  };

  const fillExampleData = () => {
    setFromAddress(generateRandomAddress());
    setToAddress(generateRandomAddress());
    setAmount('10.5');
  };

  return (
    <Card className="bg-gradient-card border-border/50">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Send className="h-5 w-5 text-secondary" />
          Create Transaction
        </CardTitle>
        <CardDescription>
          Send LDB tokens between addresses on the blockchain
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* From Address */}
          <div className="space-y-2">
            <Label htmlFor="from-address" className="text-sm font-medium">
              From Address *
            </Label>
            <div className="relative">
              <Wallet className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                id="from-address"
                type="text"
                placeholder="0x... sender address"
                value={fromAddress}
                onChange={(e) => setFromAddress(e.target.value)}
                className="pl-10 font-mono text-sm"
                disabled={transactionMutation.isPending}
                required
              />
            </div>
          </div>

          {/* Arrow Separator */}
          <div className="flex justify-center">
            <div className="flex items-center justify-center w-10 h-10 bg-primary/10 rounded-full">
              <ArrowRight className="h-5 w-5 text-primary" />
            </div>
          </div>

          {/* To Address */}
          <div className="space-y-2">
            <Label htmlFor="to-address" className="text-sm font-medium">
              To Address *
            </Label>
            <div className="relative">
              <Wallet className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                id="to-address"
                type="text"
                placeholder="0x... recipient address"
                value={toAddress}
                onChange={(e) => setToAddress(e.target.value)}
                className="pl-10 font-mono text-sm"
                disabled={transactionMutation.isPending}
                required
              />
            </div>
          </div>

          {/* Amount */}
          <div className="space-y-2">
            <Label htmlFor="amount" className="text-sm font-medium">
              Amount (LDB) *
            </Label>
            <div className="relative">
              <Input
                id="amount"
                type="number"
                step="0.01"
                min="0"
                placeholder="0.00"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="text-sm"
                disabled={transactionMutation.isPending}
                required
              />
              <span className="absolute right-3 top-3 text-sm text-muted-foreground">
                LDB
              </span>
            </div>
          </div>

          <Separator />

          {/* Action Buttons */}
          <div className="space-y-3">
            <div className="flex gap-2">
              <Button
                type="submit"
                disabled={transactionMutation.isPending}
                variant="crypto"
                className="flex-1"
              >
                {transactionMutation.isPending ? (
                  <>Processing...</>
                ) : (
                  <>
                    <Send className="h-4 w-4 mr-2" />
                    Create Transaction
                  </>
                )}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={fillExampleData}
                disabled={transactionMutation.isPending}
              >
                Example
              </Button>
            </div>

            {/* Status Messages */}
            {transactionMutation.isPending && (
              <div className="p-3 bg-secondary/10 border border-secondary/30 rounded-lg">
                <div className="flex items-center gap-2 text-sm text-secondary">
                  <div className="w-4 h-4 border-2 border-secondary border-t-transparent rounded-full animate-spin" />
                  <span className="font-medium">Creating transaction...</span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Adding your transaction to the pending pool.
                </p>
              </div>
            )}

            {transactionMutation.isError && (
              <div className="p-3 bg-destructive/10 border border-destructive/30 rounded-lg">
                <div className="flex items-center gap-2 text-sm text-destructive">
                  <AlertCircle className="h-4 w-4" />
                  <span className="font-medium">Transaction failed</span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Please check your inputs and try again.
                </p>
              </div>
            )}

            {transactionMutation.isSuccess && (
              <div className="p-3 bg-secondary/10 border border-secondary/30 rounded-lg">
                <div className="flex items-center gap-2 text-sm text-secondary">
                  <CheckCircle className="h-4 w-4" />
                  <span className="font-medium">Transaction created successfully!</span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Your transaction is now pending. Mine a block to confirm it.
                </p>
              </div>
            )}
          </div>

          {/* Transaction Info */}
          <div className="p-3 bg-primary/5 border border-primary/20 rounded-lg">
            <h4 className="text-sm font-medium text-primary mb-2">Transaction Information</h4>
            <ul className="text-xs text-muted-foreground space-y-1">
              <li>• Transactions are cryptographically signed for security</li>
              <li>• All transactions go to a pending pool until mined</li>
              <li>• Mining confirms transactions and adds them to blocks</li>
              <li>• Each transaction gets a unique hash identifier</li>
            </ul>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}