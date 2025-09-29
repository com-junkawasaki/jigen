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
    <div class="space-y-6 sm:space-y-8">
      {/* Control Panel */}
      <div class="bg-gray-800 rounded-lg p-4 sm:p-6 shadow-lg">
        <h3 class="text-lg sm:text-xl font-bold mb-4 text-purple-400">🎛️ Profiling Controls</h3>
        <div class="grid grid-cols-1 sm:flex sm:gap-4 gap-3 mb-4">
          <button
            onClick={startRecording}
            disabled={isRecording}
            class={`px-4 py-2 rounded font-medium text-sm sm:text-base ${
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
            class="px-4 py-2 rounded font-medium text-sm sm:text-base bg-red-600 hover:bg-red-500 text-white disabled:bg-gray-600 disabled:text-gray-400 disabled:cursor-not-allowed"
          >
            ⏹️ Stop Recording
          </button>

          <button
            onClick={loadTraceData}
            class="px-4 py-2 rounded font-medium text-sm sm:text-base bg-blue-600 hover:bg-blue-500 text-white"
          >
            🔄 Refresh Data
          </button>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 sm:gap-4">
          <div class="bg-gray-700 rounded p-3 text-center sm:text-left">
            <div class="text-gray-400 text-xs sm:text-sm mb-1">Status</div>
            <div class={`font-mono text-sm sm:text-base ${isRecording ? 'text-red-400' : 'text-green-400'}`}>
              {isRecording ? 'RECORDING' : 'READY'}
            </div>
          </div>
          <div class="bg-gray-700 rounded p-3 text-center sm:text-left">
            <div class="text-gray-400 text-xs sm:text-sm mb-1">Active Traces</div>
            <div class="font-mono text-blue-400 text-sm sm:text-base">{traceData.length}</div>
          </div>
          <div class="bg-gray-700 rounded p-3 text-center sm:text-left">
            <div class="text-gray-400 text-xs sm:text-sm mb-1">Total Duration</div>
            <div class="font-mono text-purple-400 text-sm sm:text-base">
              {traceData.reduce((sum, trace) => sum + trace.duration_ms, 0).toFixed(1)}ms
            </div>
          </div>
        </div>
      </div>

      {/* Flame Graph */}
      <div class="bg-gray-800 rounded-lg p-4 sm:p-6 shadow-lg">
        <h3 class="text-lg sm:text-xl font-bold mb-4 text-orange-400">🔥 Flame Graph</h3>
        <div class="bg-gray-900 rounded p-4 min-h-[250px] sm:min-h-[300px] overflow-x-auto">
          <FlameGraph traces={traceData} onSelectTrace={setSelectedTrace} />
        </div>
      </div>

      {/* Trace Details */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 sm:gap-6 lg:gap-8">
        <div class="bg-gray-800 rounded-lg p-4 sm:p-6 shadow-lg">
          <h3 class="text-lg sm:text-xl font-bold mb-4 text-cyan-400">📋 Trace List</h3>
          <div class="space-y-2 max-h-80 sm:max-h-96 overflow-y-auto">
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
                <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-2">
                  <span class="font-medium text-sm sm:text-base">{trace.name}</span>
                  <span class="text-sm text-gray-400 font-mono">{trace.duration_ms.toFixed(2)}ms</span>
                </div>
                <div class="text-xs text-gray-500">
                  Thread: {trace.thread_id} |
                  Memory: {trace.memory_usage_kb ? `${trace.memory_usage_kb}KB` : 'N/A'}
                </div>
              </div>
            ))}
          </div>
        </div>

        <div class="bg-gray-800 rounded-lg p-4 sm:p-6 shadow-lg">
          <h3 class="text-lg sm:text-xl font-bold mb-4 text-green-400">🔍 Trace Details</h3>
          {selectedTrace ? (
            <div class="space-y-3 sm:space-y-4">
              <div>
                <label class="block text-xs sm:text-sm text-gray-400 mb-1">Function Name</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-sm break-all">{selectedTrace.name}</div>
              </div>

              <div>
                <label class="block text-xs sm:text-sm text-gray-400 mb-1">Duration</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-green-400 text-sm">
                  {selectedTrace.duration_ms.toFixed(3)} ms
                </div>
              </div>

              <div>
                <label class="block text-xs sm:text-sm text-gray-400 mb-1">Memory Usage</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-blue-400 text-sm">
                  {selectedTrace.memory_usage_kb ? `${selectedTrace.memory_usage_kb} KB` : 'Not measured'}
                </div>
              </div>

              <div>
                <label class="block text-xs sm:text-sm text-gray-400 mb-1">Thread ID</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-purple-400 text-sm">
                  {selectedTrace.thread_id}
                </div>
              </div>

              <div>
                <label class="block text-xs sm:text-sm text-gray-400 mb-1">Start Time</label>
                <div class="font-mono bg-gray-900 p-2 rounded text-xs sm:text-sm break-all">
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
      <div class="bg-gray-800 rounded-lg p-4 sm:p-6 shadow-lg">
        <h3 class="text-lg sm:text-xl font-bold mb-4 text-red-400">🧠 Memory Analysis</h3>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 sm:gap-4">
          <div class="text-center p-4 bg-gray-700 rounded-lg">
            <div class="text-xl sm:text-2xl font-bold text-blue-400 mb-2">
              {Math.max(...traceData.map(t => t.memory_usage_kb || 0))} KB
            </div>
            <div class="text-xs sm:text-sm text-gray-400">Peak Memory</div>
          </div>
          <div class="text-center p-4 bg-gray-700 rounded-lg">
            <div class="text-xl sm:text-2xl font-bold text-green-400 mb-2">
              {(traceData.reduce((sum, t) => sum + (t.memory_usage_kb || 0), 0) / Math.max(traceData.length, 1)).toFixed(0)} KB
            </div>
            <div class="text-xs sm:text-sm text-gray-400">Average Memory</div>
          </div>
          <div class="text-center p-4 bg-gray-700 rounded-lg sm:col-span-2 lg:col-span-1">
            <div class="text-xl sm:text-2xl font-bold text-yellow-400 mb-2">
              {traceData.filter(t => t.memory_usage_kb && t.memory_usage_kb > 1000).length}
            </div>
            <div class="text-xs sm:text-sm text-gray-400">High Memory Traces</div>
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

    // Responsive width based on container
    const containerWidth = container.node()?.parentElement?.clientWidth || 800;
    const width = Math.max(containerWidth - 40, 400); // Minimum width
    const height = Math.max(traces.length * 35, 200); // Dynamic height based on traces
    const barHeight = 30;
    const paddingLeft = 100; // Space for labels

    const svg = container
      .append('svg')
      .attr('width', width)
      .attr('height', height)
      .attr('viewBox', `0 0 ${width} ${height}`);

    const maxDuration = d3.max(traces, d => d.duration_ms) || 1;
    const xScale = d3.scaleLinear().domain([0, maxDuration]).range([paddingLeft, width - 20]);

    // Add background grid
    const gridLines = svg.selectAll('.grid-line')
      .data(d3.range(0, maxDuration + 1, maxDuration / 5))
      .enter()
      .append('line')
      .attr('class', 'grid-line')
      .attr('x1', d => xScale(d))
      .attr('y1', 0)
      .attr('x2', d => xScale(d))
      .attr('y2', height)
      .attr('stroke', '#374151')
      .attr('stroke-width', 1)
      .attr('opacity', 0.3);

    const bars = svg.selectAll('rect')
      .data(traces)
      .enter()
      .append('rect')
      .attr('x', paddingLeft)
      .attr('y', (d, i) => i * (barHeight + 5) + 5)
      .attr('width', d => Math.max(xScale(d.duration_ms) - paddingLeft, 2))
      .attr('height', barHeight)
      .attr('fill', (d, i) => d3.schemeCategory10[i % 10])
      .attr('rx', 3)
      .style('cursor', 'pointer')
      .on('click', (event, d) => onSelectTrace(d))
      .append('title')
      .text(d => `${d.name}: ${d.duration_ms.toFixed(2)}ms`);

    const labels = svg.selectAll('text.label')
      .data(traces)
      .enter()
      .append('text')
      .attr('class', 'label')
      .attr('x', 5)
      .attr('y', (d, i) => i * (barHeight + 5) + barHeight / 2 + 8)
      .attr('fill', 'white')
      .attr('font-size', '11px')
      .attr('font-family', 'monospace')
      .style('pointer-events', 'none')
      .text(d => d.name.length > 12 ? d.name.substring(0, 12) + '...' : d.name);

    const timeLabels = svg.selectAll('text.time')
      .data(traces)
      .enter()
      .append('text')
      .attr('class', 'time')
      .attr('x', d => xScale(d.duration_ms) + 5)
      .attr('y', (d, i) => i * (barHeight + 5) + barHeight / 2 + 8)
      .attr('fill', '#9CA3AF')
      .attr('font-size', '10px')
      .attr('font-family', 'monospace')
      .style('pointer-events', 'none')
      .text(d => `${d.duration_ms.toFixed(1)}ms`);

  }, [traces, onSelectTrace]);

  return (
    <div class="w-full">
      <div id="flame-graph" class="w-full min-h-[200px] bg-gray-900 rounded border border-gray-600"></div>
      {traces.length === 0 && (
        <div class="text-center text-gray-500 py-8">
          No trace data available. Click "Start Recording" to begin profiling.
        </div>
      )}
    </div>
  );
}
