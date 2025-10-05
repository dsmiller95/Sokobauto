import CosmosGraph from '@/features/graph/components/CosmosGraph';
import { createFileRoute } from '@tanstack/react-router'
import React from 'react'


async function getGraphData(): Promise<GraphData> {
  const response = await fetch('/graph/data-json')
  return await response.json()
}

export const Route = createFileRoute('/graph/view')({
  component: RouteComponent,
})

function RouteComponent() {
  const [graphData, setGraphData] = React.useState<GraphData | null>(null);

  React.useEffect(() => {
    getGraphData().then((data) => {
      setGraphData(data);
    });
  }, []);

  if(!graphData) {
    return <div>Loading graph data...</div>;
  }

  return <CosmosGraph data={graphData} />;
}
