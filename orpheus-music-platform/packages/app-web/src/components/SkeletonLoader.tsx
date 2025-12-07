/**
 * Skeleton Loader - Animated placeholder for loading content
 * Memoized for performance during lazy loading
 */

import { memo } from 'react';
import { cn } from '../lib/utils';

export interface SkeletonLoaderProps {
  /** Variant of skeleton to show */
  variant?: 'compose' | 'record' | 'mix' | 'master' | 'practice' | 'distribute' | 'generic';
}

const skeletonBase = 'bg-muted rounded relative overflow-hidden before:absolute before:top-0 before:-left-full before:h-full before:w-full before:bg-gradient-to-r before:from-transparent before:via-background before:to-transparent before:animate-shimmer';

export const SkeletonLoader = memo(function SkeletonLoader({ variant = 'generic' }: SkeletonLoaderProps) {
  switch (variant) {
    case 'compose':
      return <ComposeSkeleton />;
    case 'record':
      return <RecordSkeleton />;
    case 'mix':
      return <MixSkeleton />;
    case 'master':
      return <MasterSkeleton />;
    case 'practice':
      return <PracticeSkeleton />;
    case 'distribute':
      return <DistributeSkeleton />;
    default:
      return <GenericSkeleton />;
  }
});

function ComposeSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Toolbar */}
      <div className="flex gap-3">
        <div className={cn(skeletonBase, 'h-8 w-24')} />
        <div className={cn(skeletonBase, 'h-8 w-24')} />
        <div className={cn(skeletonBase, 'h-8 w-24')} />
      </div>
      {/* Tab Editor */}
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg flex-1')} />
      {/* Controls */}
      <div className="flex gap-3">
        <div className={cn(skeletonBase, 'h-8 w-24')} />
        <div className={cn(skeletonBase, 'h-8 w-24')} />
      </div>
    </div>
  );
}

function RecordSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Header */}
      <div className={cn(skeletonBase, 'h-10')} />
      {/* Waveform */}
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      {/* Tracks */}
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
    </div>
  );
}

function MixSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Mixer Channels */}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-4">
        <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
        <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
        <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
        <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      </div>
    </div>
  );
}

function MasterSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Meters */}
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      {/* Mastering Chain */}
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      {/* Export Options */}
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
    </div>
  );
}

function PracticeSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Speed Trainer */}
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      {/* Loop Section */}
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
      {/* Progress */}
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
    </div>
  );
}

function DistributeSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      {/* Platform Cards */}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(250px,1fr))] gap-4">
        <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
        <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
        <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
      </div>
    </div>
  );
}

function GenericSkeleton() {
  return (
    <div className="flex flex-col gap-4 p-6 h-full">
      <div className={cn(skeletonBase, 'h-4 w-[60%]')} />
      <div className={cn(skeletonBase, 'h-4 w-full')} />
      <div className={cn(skeletonBase, 'h-[200px] rounded-lg')} />
      <div className={cn(skeletonBase, 'h-[120px] rounded-lg')} />
    </div>
  );
}
