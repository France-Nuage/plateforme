<template>
  <div id="chart-pie" />
  <div class="space-y-2 mt-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-3 h-3 rounded-full mr-2" style="background-color: rgb(99, 102, 241);"></div>
        <span class="text-sm text-gray-600 dark:text-gray-400">Compute Engine</span>
      </div>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-300">1 250,80 $US (45%)</span>
    </div>
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-3 h-3 rounded-full mr-2" style="background-color: rgb(139, 92, 246);"></div>
        <span class="text-sm text-gray-600 dark:text-gray-400">Cloud Storage</span>
      </div>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-300">520,30 $US (19%)</span>
    </div>
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-3 h-3 rounded-full mr-2" style="background-color: rgb(236, 72, 153);"></div>
        <span class="text-sm text-gray-600 dark:text-gray-400">Cloud SQL</span>
      </div>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-300">480,50 $US (17%)</span>
    </div>
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-3 h-3 rounded-full mr-2" style="background-color: rgb(244, 63, 94);"></div>
        <span class="text-sm text-gray-600 dark:text-gray-400">Network</span>
      </div>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-300">320,20 $US (12%)</span>
    </div>
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-3 h-3 rounded-full mr-2" style="background-color: rgb(245, 158, 11);"></div>
        <span class="text-sm text-gray-600 dark:text-gray-400">Other</span>
      </div>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-300">195,40 $US (7%)</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import ApexCharts from "apexcharts";

const options = {
  series: [44, 55, 13, 33],
  chart: {
    type: 'donut',
    width: 380,
  },
  chartOptions: {
    labels: ['Apple', 'Mango', 'Orange', 'Watermelon'],
  },
  legend: {
    show: false,
    position: 'bottom',
    formatter: function(seriesName: string, opts: any) {
      return [seriesName, " - ", opts.w.globals.series[opts.seriesIndex]]
    }
  },
  stroke: {
    width: 0
  },
  plotOptions: {
    // pie: {
    //   startAngle: -90,
    //   endAngle: 90,
    //   offsetY: 10
    // }
  },
}

// var options = {
//   series: [44, 55, 13, 43, 22],
//   chart: {
//     width: 380,
//     type: 'pie',
//   },
//   labels: ['Team A', 'Team B', 'Team C', 'Team D', 'Team E'],
//   responsive: [{
//     breakpoint: 480,
//     options: {
//       chart: {
//         width: 200
//       },
//       legend: {
//         position: 'bottom'
//       }
//     }
//   }]
// };

onMounted(() => {
  if (import.meta.browser) {
    const chart1 = new ApexCharts(document.querySelector("#chart-pie"), options);
    chart1.render();
  }
})
</script>

<style scoped>

</style>