import { useEffect, useState } from 'react'
import { list } from './services/instance-service'
import { InstanceInfo } from './protocol/controlplane'

function App() {
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  useEffect(() => {
    list().then(setInstances);
  }, [setInstances])

  return (
    <table>
      <thead>
        <tr>
          <th>id</th>
          <th>status</th>
          <th>maxCpuCores</th>
          <th>cpuUsagePercent</th>
          <th>maxMemoryBytes</th>
          <th>memoryUsageBytes</th>

        </tr>
      </thead>
      <tbody>
        {instances.map((instance) => (
          <tr key={instance.id}>
            <td>{instance.id}</td>
            <td>{instance.status}</td>
            <td>{instance.maxCpuCores}</td>
            <td>{instance.cpuUsagePercent}</td>
            <td>{instance.maxMemoryBytes}</td>
            <td>{instance.memoryUsageBytes}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}

export default App
