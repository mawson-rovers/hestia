(function () { // prevent leaking into global scope
    const ctx = document.getElementById('temperature-chart');
    const initialDurationMins = location.hash && location.hash.match(/d\d+/) ?
        location.hash.match(/d(\d+)/)[1] : 30; // default to 30 mins

    function getChartData(data) {
        let sensor_ids = Object.keys(data)
            .filter(id => data[id].length); // exclude empty sensors
        return {
            datasets: sensor_ids.map(function (id) {
                return {
                    label: id,
                    data: data[id],
                    borderWidth: 1,
                    xAxisID: 'x',
                    yAxisID: id === 'heater' ? 'y2' : 'y1',
                };
            }),
        };
    }

    function minsToMillis(durationMins) {
        return durationMins * 60000;
    }

    function updateChartDuration(chart) {
        const now = new Date();
        const durationMillis = minsToMillis(chart.options.durationMins);
        chart.options.scales.x.min = new Date(now.getTime() - durationMillis);
        chart.options.scales.x.max = now;
    }

    function updateChartData(chart, newData) {
        chart.data.datasets.forEach(dataset => {
            if (dataset.label in newData && newData[dataset.label].length) {
                console.log("before: " + dataset.data.length);
                newData[dataset.label].forEach(item => {
                    dataset.data.shift();
                    dataset.data.push(item);
                });
                console.log("after: " + dataset.data.length);
            }
        });
    }

    fetch('/api/log_data')
        .then(response => response.json())
        .then(function (data) {
            const chart = new Chart(ctx, {
                type: 'line',
                responsive: true,
                maintainAspectRatio: false,
                data: getChartData(data),
                durationMins: initialDurationMins,
                options: {
                    scales: {
                        x: {
                            type: 'time',
                            time: {
                                unit: 'minute',
                                // displayFormats: {
                                //     second: 'HH:mm:ss',
                                //     minute: 'HH:mm:ss',
                                // },
                            },
                            min: new Date(new Date().getTime() - minsToMillis(initialDurationMins)),
                            max: new Date(),
                        },
                        y1: {
                            beginAtZero: true,
                            suggestedMax: 80.0,
                            title: {
                                display: true,
                                text: 'Temperature (°C)',
                            },
                        },
                        y2: {
                            beginAtZero: true,
                            suggestedMax: 255,
                            title: {
                                display: true,
                                text: 'Power level (0-255)',
                            },
                            position: 'right',
                            grid: {
                                drawOnChartArea: false, // don't show grid lines for secondary axis
                            },
                        }
                    },
                    elements: {
                        point: {
                            radius: 0.5,
                        }
                    },
                    plugins: {
                        legend: {
                            position: "right",
                        }
                    }
                },
            });

            // set heater series to red
            const heaterDataset = chart.data.datasets.find(d => d.label === 'heater')
            if (heaterDataset) {
                heaterDataset.fill = true;
                heaterDataset.borderColor = 'rgb(255, 99, 132)';
                heaterDataset.backgroundColor = 'rgb(255, 99, 132, 0.5)';
            }

            // load new data at regular intervals
            window.setInterval(() => {
                fetch('/api/data')
                    .then((response) => response.json())
                    .then((newData) => {
                        updateChartData(chart, newData);
                        updateChartDuration(chart);
                        chart.update();
                    })
            }, 5000);

            // click handlers for chart duration
            document.querySelectorAll(".duration-selector a").forEach(function (el) {
                let durationMins = el.getAttribute("data-duration-mins")
                el.addEventListener('click', (ev) => {
                    chart.options.durationMins = durationMins;
                    updateChartDuration(chart);
                    chart.update();

                    // add #d10 or #d30 to URL for state persistence
                    if (history.pushState) {
                        history.pushState(null, null, '#d' + durationMins);
                    } else {
                        location.hash = 'd' + durationMins;
                    }

                    // prevent links jumping the page around
                    ev.stopPropagation();
                    ev.preventDefault();
                });
            });

            window.chart = chart;
        });
})();
