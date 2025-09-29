import { useState, useEffect } from 'react';

interface ChartData {
  name: string;
  data: Array<{ [key: string]: number }>;
}

interface BenchmarkSummaryProps {
  data: {
    sceneParsing: ChartData;
    sceneGraph: ChartData;
    physics: ChartData;
  };
}

export default function BenchmarkSummary({ data }: BenchmarkSummaryProps) {
  const [summary, setSummary] = useState({
    totalBenchmarks: 0,
    averageTime: 0,
    bestPerformance: '',
    worstPerformance: '',
    trend: 'stable'
  });

  useEffect(() => {
    // Calculate summary statistics
    const allData = [...data.sceneParsing.data, ...data.sceneGraph.data, ...data.physics.data];
    const times = allData.map(d => d.time).filter(t => t > 0);

    if (times.length > 0) {
      const avg = times.reduce((a, b) => a + b, 0) / times.length;
      const min = Math.min(...times);
      const max = Math.max(...times);

      setSummary({
        totalBenchmarks: times.length,
        averageTime: avg,
        bestPerformance: `${min.toFixed(2)}ms`,
        worstPerformance: `${max.toFixed(2)}ms`,
        trend: 'stable' // In a real implementation, you'd calculate trends
      });
    }
  }, [data]);

  return (
    <div class="space-y-4">
      <div class="grid grid-cols-2 gap-4">
        <div class="text-center">
          <div class="text-2xl font-bold text-blue-400">{summary.totalBenchmarks}</div>
          <div class="text-sm text-gray-400">Total Benchmarks</div>
        </div>
        <div class="text-center">
          <div class="text-2xl font-bold text-green-400">{summary.averageTime.toFixed(2)}ms</div>
          <div class="text-sm text-gray-400">Average Time</div>
        </div>
      </div>

      <div class="space-y-2">
        <div class="flex justify-between">
          <span class="text-gray-400">Best Performance:</span>
          <span class="text-green-400 font-mono">{summary.bestPerformance}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Worst Performance:</span>
          <span class="text-red-400 font-mono">{summary.worstPerformance}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">Performance Trend:</span>
          <span class={`font-mono ${
            summary.trend === 'improving' ? 'text-green-400' :
            summary.trend === 'degrading' ? 'text-red-400' :
            'text-yellow-400'
          }`}>
            {summary.trend}
          </span>
        </div>
      </div>

      <div class="mt-4">
        <h4 class="text-lg font-semibold mb-2">Benchmark Categories</h4>
        <div class="space-y-1">
          <div class="flex justify-between text-sm">
            <span>Scene Parsing</span>
            <span class="text-blue-400">{data.sceneParsing.data.length} tests</span>
          </div>
          <div class="flex justify-between text-sm">
            <span>SceneGraph Loading</span>
            <span class="text-green-400">{data.sceneGraph.data.length} tests</span>
          </div>
          <div class="flex justify-between text-sm">
            <span>Physics Simulation</span>
            <span class="text-purple-400">{data.physics.data.length} tests</span>
          </div>
        </div>
      </div>
    </div>
  );
}
