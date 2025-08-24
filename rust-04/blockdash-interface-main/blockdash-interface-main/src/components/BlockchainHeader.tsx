import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Blocks, Zap, Globe, Github, ExternalLink } from 'lucide-react';

export function BlockchainHeader() {
  return (
    <div className="relative overflow-hidden">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-hero" />
      <div className="absolute inset-0 bg-gradient-to-r from-transparent via-primary/5 to-transparent" />
      
      {/* Animated blockchain flow effect */}
      <div className="absolute inset-0 opacity-10">
        <div className="animate-blockchain w-8 h-8 bg-primary rounded-full absolute top-8 left-0" />
        <div className="animate-blockchain w-6 h-6 bg-secondary rounded-full absolute top-12 left-0" style={{ animationDelay: '2s' }} />
        <div className="animate-blockchain w-4 h-4 bg-accent rounded-full absolute top-16 left-0" style={{ animationDelay: '4s' }} />
      </div>

      <Card className="relative bg-gradient-card/80 backdrop-blur-sm border-border/50 shadow-glow">
        <CardContent className="p-8">
          <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6">
            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <div className="p-3 bg-primary/10 rounded-xl">
                  <Blocks className="h-8 w-8 text-primary" />
                </div>
                <div>
                  <h1 className="text-3xl font-bold bg-gradient-primary bg-clip-text text-transparent">
                    LedgerDB Interface
                  </h1>
                  <p className="text-muted-foreground">
                    High-Performance Blockchain Ledger Database
                  </p>
                </div>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="outline" className="border-primary/30 text-primary">
                  <Zap className="h-3 w-3 mr-1" />
                  Rust Backend
                </Badge>
                <Badge variant="outline" className="border-secondary/30 text-secondary">
                  <Globe className="h-3 w-3 mr-1" />
                  Real-time WebSocket
                </Badge>
                <Badge variant="outline" className="border-accent/30 text-accent">
                  <Blocks className="h-3 w-3 mr-1" />
                  Proof of Work
                </Badge>
              </div>

              <p className="text-sm text-muted-foreground max-w-2xl">
                A production-ready blockchain implementation with embedded database, 
                WebSocket real-time updates, and enterprise-grade architecture. 
                Connect to your Rust backend running on localhost:3000.
              </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-3 lg:flex-col xl:flex-row">
              <Button 
                variant="hero-primary" 
                onClick={() => window.open('https://github.com/rjaysolamo/blockchain-learn', '_blank')}
              >
                <Github className="h-4 w-4 mr-2" />
                View Source
                <ExternalLink className="h-3 w-3 ml-2" />
              </Button>
              <Button 
                variant="hero-secondary"
                onClick={() => {
                  const element = document.getElementById('dashboard');
                  element?.scrollIntoView({ behavior: 'smooth' });
                }}
              >
                Explore Dashboard
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}