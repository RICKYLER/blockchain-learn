import { BlockchainHeader } from '@/components/BlockchainHeader';
import { BlockchainStats } from '@/components/BlockchainStats';
import { BlocksList } from '@/components/BlocksList';
import { MiningControls } from '@/components/MiningControls';
import { TransactionForm } from '@/components/TransactionForm';
import { PendingTransactions } from '@/components/PendingTransactions';
import { WebSocketStatus } from '@/components/WebSocketStatus';

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      {/* Header Section */}
      <div className="container mx-auto px-4 py-8">
        <BlockchainHeader />
      </div>

      {/* Dashboard Section */}
      <div id="dashboard" className="container mx-auto px-4 pb-12">
        <div className="space-y-8">
          {/* WebSocket Status */}
          <WebSocketStatus />

          {/* Blockchain Statistics */}
          <div>
            <h2 className="text-2xl font-bold mb-6 flex items-center gap-2">
              <span className="bg-gradient-primary bg-clip-text text-transparent">
                Blockchain Overview
              </span>
            </h2>
            <BlockchainStats />
          </div>

          {/* Main Dashboard Grid */}
          <div className="grid grid-cols-1 xl:grid-cols-3 gap-8">
            {/* Left Column - Blocks */}
            <div className="xl:col-span-2 space-y-8">
              <BlocksList />
            </div>

            {/* Right Column - Controls & Transactions */}
            <div className="space-y-8">
              <MiningControls />
              <TransactionForm />
              <PendingTransactions />
            </div>
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="border-t border-border/50 bg-gradient-card/50">
        <div className="container mx-auto px-4 py-8">
          <div className="text-center text-sm text-muted-foreground">
            <p>
              LedgerDB Interface - Built with React, TypeScript, and Tailwind CSS
            </p>
            <p className="mt-2">
              Connect to your Rust blockchain backend on{' '}
              <code className="bg-background/50 px-2 py-1 rounded">localhost:3000</code>
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Index;
