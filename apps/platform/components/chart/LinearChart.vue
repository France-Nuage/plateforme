<template>
  <div id="chart-area"></div>
</template>

<script setup lang="ts">
import ApexCharts from 'apexcharts'

interface Props {
  data: Array<any>
}

const props = defineProps<Props>()

const generateDayWiseTimeSeries = (baseval, count, yrange) => {
  var i = 0;
  var series = [];
  while (i < count) {
    var x = baseval;
    var y =
      Math.floor(Math.random() * (yrange.max - yrange.min + 1)) + yrange.min;

    series.push([x, y]);
    baseval += 86400000;
    i++;
  }
  return series;
}

var data = generateDayWiseTimeSeries(new Date("22 Apr 2017").getTime(), 12, {
  min: 10000,
  max: 20000
});

var options1 = {
  chart: {
    id: "chart2",
    type: "area",
    height: 360,
    foreColor: "#ccc",
    toolbar: {
      autoSelected: "pan",
      show: false
    }
  },
  colors: ["#2DB26F"],
  stroke: {
    width: 3
  },
  grid: {
    borderColor: "#555",
    clipMarkers: false,
    yaxis: {
      lines: {
        show: false
      }
    }
  },
  dataLabels: {
    enabled: false
  },
  fill: {
    gradient: {
      enabled: true,
      opacityFrom: 0.55,
      opacityTo: 0
    }
  },
  markers: {
    size: 2,
    // colors: ["#000524"],
    strokeColor: "#2DB26F",
    strokeWidth: 2
  },
  series: [
    {
      data: data
    }
  ],
  tooltip: {
    theme: "dark"
  },
  xaxis: {
    type: "datetime"
  },
  yaxis: {
    min: 0,
    tickAmount: 4
  }
};

onMounted(() => {
  if (import.meta.browser) {
    const chart1 = new ApexCharts(document.querySelector("#chart-area"), options1);
    chart1.render();
  }
})
</script>
