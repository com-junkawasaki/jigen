import { useEffect, useRef } from 'react';
import Chart from 'chart.js/auto';

interface ChartData {
  name: string;
  data: Array<{ [key: string]: number }>;
}

interface PerformanceChartsProps {
  sceneParsing: ChartData;
  sceneGraph: ChartData;
  physics: ChartData;
}

export default function PerformanceCharts({ sceneParsing, sceneGraph, physics }: PerformanceChartsProps) {
  const sceneParsingRef = useRef<HTMLCanvasElement>(null);
  const sceneGraphRef = useRef<HTMLCanvasElement>(null);
  const physicsRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!sceneParsingRef.current || !sceneGraphRef.current || !physicsRef.current) return;

    // Scene Parsing Chart
    const sceneParsingChart = new Chart(sceneParsingRef.current, {
      type: 'line',
      data: {
        labels: sceneParsing.data.map(d => d.nodes?.toString() || d.steps?.toString()),
        datasets: [{
          label: 'Parsing Time (ms)',
          data: sceneParsing.data.map(d => d.time),
          borderColor: 'rgb(59, 130, 246)',
          backgroundColor: 'rgba(59, 130, 246, 0.1)',
          tension: 0.4,
          fill: true,
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Scene Parsing Performance',
            font: {
              size: window.innerWidth < 640 ? 12 : 14
            }
          },
          legend: {
            display: window.innerWidth < 640 ? false : true
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            title: {
              display: true,
              text: 'Time (ms)',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          },
          x: {
            title: {
              display: true,
              text: 'Node Count',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          }
        }
      }
    });

    // SceneGraph Loading Chart
    const sceneGraphChart = new Chart(sceneGraphRef.current, {
      type: 'bar',
      data: {
        labels: sceneGraph.data.map(d => d.nodes?.toString() || d.steps?.toString()),
        datasets: [{
          label: 'Loading Time (ms)',
          data: sceneGraph.data.map(d => d.time),
          backgroundColor: 'rgba(34, 197, 94, 0.8)',
          borderColor: 'rgb(34, 197, 94)',
          borderWidth: 1,
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'SceneGraph Loading Performance',
            font: {
              size: window.innerWidth < 640 ? 12 : 14
            }
          },
          legend: {
            display: window.innerWidth < 640 ? false : true
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            title: {
              display: true,
              text: 'Time (ms)',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          },
          x: {
            title: {
              display: true,
              text: 'Node Count',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          }
        }
      }
    });

    // Physics Simulation Chart
    const physicsChart = new Chart(physicsRef.current, {
      type: 'scatter',
      data: {
        datasets: [{
          label: 'Physics Steps',
          data: physics.data.map(d => ({
            x: d.steps || d.nodes || 0,
            y: d.time
          })),
          backgroundColor: 'rgba(147, 51, 234, 0.8)',
          borderColor: 'rgb(147, 51, 234)',
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          title: {
            display: true,
            text: 'Physics Simulation Performance',
            font: {
              size: window.innerWidth < 640 ? 12 : 14
            }
          },
          legend: {
            display: window.innerWidth < 640 ? false : true
          }
        },
        scales: {
          y: {
            beginAtZero: true,
            title: {
              display: true,
              text: 'Time (ms)',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          },
          x: {
            title: {
              display: true,
              text: 'Steps',
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            },
            ticks: {
              font: {
                size: window.innerWidth < 640 ? 10 : 12
              }
            }
          }
        }
      }
    });

    return () => {
      sceneParsingChart.destroy();
      sceneGraphChart.destroy();
      physicsChart.destroy();
    };
  }, [sceneParsing, sceneGraph, physics]);

  return (
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 sm:gap-6">
      <div class="bg-gray-700 rounded p-3 sm:p-4 min-h-[300px] sm:min-h-[350px]">
        <canvas ref={sceneParsingRef} class="w-full h-full"></canvas>
      </div>
      <div class="bg-gray-700 rounded p-3 sm:p-4 min-h-[300px] sm:min-h-[350px]">
        <canvas ref={sceneGraphRef} class="w-full h-full"></canvas>
      </div>
      <div class="bg-gray-700 rounded p-3 sm:p-4 min-h-[300px] sm:min-h-[350px] sm:col-span-2 lg:col-span-1">
        <canvas ref={physicsRef} class="w-full h-full"></canvas>
      </div>
    </div>
  );
}
