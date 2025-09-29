import { useState, useEffect } from 'react';
import * as d3 from 'd3';

interface TraceData {
  name: string;
  start_time: string;
  duration_ms: number;
  memory_usage_kb: number | null;
  thread_id: string;
}

export default function ProfilerDashboard() {
  const [traceData, setTraceData] = useState<TraceData[]>([]);
  const [selectedTrace, setSelectedTrace] = useState<TraceData | null>(null);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    loadTraceData();
  }, []);

  const loadTraceData = async () => {
    try {
      // In a real implementation, this would fetch from the Rust profiler
      const mockTraces: TraceData[] = [
        {
          name: 'scene_parsing',
          start_time: new Date(Date.now() - 5000).toISOString(),
          duration_ms: 245.3,
          memory_usage_kb: 1250,
          thread_id: 'main'
        },
        {
          name: 'scene_graph_build',
          start_time: new Date(Date.now() - 4500).toISOString(),
          duration_ms: 123.7,
          memory_usage_kb: 890,
          thread_id: 'main'
        },
        {
          name: 'physics_simulation',
          start_time: new Date(Date.now() - 4300).toISOString(),
          duration_ms: 45.2,
          memory_usage_kb: 567,
          thread_id: 'physics'
        },
        {
          name: 'render_update',
          start_time: new Date(Date.now() - 4200).toISOString(),
          duration_ms: 16.7,
          memory_usage_kb: 234,
          thread_id: 'render'
        }
      ];

      setTraceData(mockTraces);
    } catch (error) {
      console.error('Failed to load trace data:', error);
    }
  };

  const startRecording = () => {
    setIsRecording(true);
    // In a real implementation, this would start the Rust profiler
  };

  const stopRecording = () => {
    setIsRecording(false);
    loadTraceData(); // Reload data after recording
  };

  return (
    <div class="space-y-8">
      {/* Control Panel */}
      <div class="bg-gray-800 rounded-lg p-6 shadow-lg">
        <h3 class="text-xl font-bold mb-4 text-purple-400">🎛️ Profiling Controls</h3>
        <div class="flex gap-4 mb-4">
          <button
            onClick={startRecording}
            disabled={isRecording}
            class={`px-4 py-2 rounded font-medium ${
              isRecording
                ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                : 'bg-green-600 hover:bg-green-500 text-white'
            }`}
          >
            {isRecording ? '🔴 Recording...' : '▶️ Start Recording'}
          </button>

          <button
            onClick={stopRecording}
            disabled={!isRecording}
            class="px-4 py-2 rounded font-medium bg-red-600 hover:bg-red-500 text-white disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed"
          >
            ⏹️ Stop Recording
          </button>

          <button
            onClick={loadTraceData}
            class="px-4 py-2 rounded font-medium bg-blue-600 hover:bg-blue-500 text-white"
          >
            🔄 Refresh Data
          </button>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
          <div class="bg-gray-700 rounded p-3">
            <div class="text-gray-400">Status</div>
            <div class={`font-mono ${isRecording ? 'text-red-400' : 'text-green-400'}`}>
              {isRecording ? 'RECORDING' : 'READY'}
            </div>
          </div>
          <div class="bg-gray-700 rounded p-3">
            <div class="text-gray-400">Active Traces</div>
            <div class="font-mono text-blue-400">{traceData.length}</div>
          </div>
          <div class="bg-gray-700 rounded p-3">
            <div class="text-gray-400">Total Duration</div>
            <div class="font-mono text-purple-400">
              {traceData.reduce((sum, trace) => sum + trace.duration_ms, 0).toFixed(1)}ms
            </div>
          </div>
        </div>
      </div>

      {/* Flame Graph */}
      <div class="bg-gray-800 rounded-lg p-6 shadow-lg">
        <h3 class="text-xl font-bold mb-4 text-orange-400">🔥 Flame Graph</h3>
        <div class="bg-gray-900 rounded p-4 min-h-[300px]">
          <FlameGraph traces={traceData} onSelectTrace={setSelectedTrace} />
        </div>
      </div>

      {/* Trace Details */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div class="bg-gray-800 rounded-lg p-6 shadow-lg">
          <h3 class="text-xl font-bold mb-4 text-cyan-400">📋 Trace List</h3>
          <div class="space-y-2 max-h-96 overflow-y-auto">
            {traceData.map((trace, index) => (
              <div
                key={index}
                onClick={() => setSelectedTrace(trace)}
                class={`p-3 rounded cursor-pointer transition-colors ${
                  selectedTrace?.name === trace.name
                    ? 'bg-blue-600 border border-blue-400'
                    : 'bg-gray-700 hover:bg-gray-600'
                }`}
              >
                <div class="flex justify-between items-center">
                  <span class="font-medium">{trace.name}</span>
                  <span class="text-sm text-gray-400">{trace.duration_ms.toFixed(2)}ms</span>
                </div>
                <div class="text-xs text-gray-500 mt-1">
                  Thread: {trace.thread_id} |
                  Memory: {trace.memory_usage_kb ? `${trace.memory_usage_kb}KB` : 'N/A'}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div class="bg-gray-800 rounded-lg p-6 shadow-lg">
          <h3 class="text-xl font-bold mb-4 text-green-400">🔍 Trace Details</h3>
          {selectedTrace ? (
            <div class="space-y-4">
              <div>
                <label class="block text-sm text-gray-400 mb-1">Function Name</label>
                <div class="font-mono bg-gray-900 p-2 rounded">{selectedTrace.name}</div>
              </div>

              <div>
                <label class="block text-sm text-gray-400 mb-1">Duration</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-green-400">
                  {selectedTrace.duration_ms.toFixed(3)} ms
                </div>
              </div>

              <div>
                <label class="block text-sm text-gray-400 mb-1">Memory Usage</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-blue-400">
                  {selectedTrace.memory_usage_kb ? `${selectedTrace.memory_usage_kb} KB` : 'Not measured'}
                </div>
              </div>

              <div>
                <label class="block text-sm text-gray-400 mb-1">Thread ID</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-purple-400">
                  {selectedTrace.thread_id}
                </div>
              </div>

              <div>
                <label class="block text-sm text-gray-400 mb-1">Start Time</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-sm">
                  {new Date(selectedTrace.start_time).toLocaleString()}
                </div>
              </div>
            </div>
          ) : (
            <div class="text-center text-gray-500 py-8">
              Select a trace from the list to view details
            </div>
          )}
        </div>
      </div>

      {/* Memory Analysis */}
      <div class="bg-gray-800 rounded-lg p-6 shadow-lg">
        <h3 class="text-xl font-bold mb-4 text-red-400">🧠 Memory Analysis</h3>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="text-center">
            <div class="text-2xl font-bold text-blue-400">
              {Math.max(...traceData.map(t => t.memory_usage_kb || 0))} KB
            </div>
            <div class="text-sm text-gray-400">Peak Memory</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-green-400">
              {(traceData.reduce((sum, t) => sum + (t.memory_usage_kb || 0), 0) / Math.max(traceData.length, 1)).toFixed(0)} KB
            </div>
            <div class="text-sm text-gray-400">Average Memory</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-yellow-400">
              {traceData.filter(t => t.memory_usage_kb && t.memory_usage_kb > 1000).length}
            </div>
            <div class="text-sm text-gray-400">High Memory Traces</div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Simple Flame Graph component
function FlameGraph({ traces, onSelectTrace }: { traces: TraceData[], onSelectTrace: (trace: TraceData) => void }) {
  useEffect(() => {
    if (!traces.length) return;

    const container = d3.select('#flame-graph');
    container.selectAll('*').remove();

    const width = 800;
    const height = 200;
    const barHeight = 30;

    const svg = container
      .append('svg')
      .attr('width', width)
      .attr('height', height);

    const maxDuration = d3.max(traces, d => d.duration_ms) || 1;
    const xScale = d3.scaleLinear().domain([0, maxDuration]).range([0, width]);

    const bars = svg.selectAll('rect')
      .data(traces)
      .enter()
      .append('rect')
      .attr('x', 0)
      .attr('y', (d, i) => i * barHeight)
      .attr('width', d => xScale(d.duration_ms))
      .attr('height', barHeight - 2)
      .attr('fill', (d, i) => d3.schemeCategory10[i % 10])
      .attr('rx', 3)
      .style('cursor', 'pointer')
      .on('click', (event, d) => onSelectTrace(d));

    const labels = svg.selectAll('text')
      .data(traces)
      .enter()
      .append('text')
      .attr('x', 5)
      .attr('y', (d, i) => i * barHeight + barHeight / 2 + 5)
      .attr('fill', 'white')
      .attr('font-size', '12px')
      .style('pointer-events', 'none')
      .text(d => `${d.name} (${d.duration_ms.toFixed(1)}ms)`);

  }, [traces, onSelectTrace]);

  return <div id="flame-graph" class="w-full overflow-x-auto"></div>;
}
