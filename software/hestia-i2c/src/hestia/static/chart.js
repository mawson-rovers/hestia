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

    function updateChartData(chart, newDatasets) {
        Object.keys(newDatasets).forEach(function (label) {
            let newData = newDatasets[label];

            let dataset = chart.data.datasets.find(ds => ds.label === label);
            if (dataset) {
                // add all new data items (API can return multiple points)
                dataset.data.push(...newData);

                // limit samples to maximum visible (30 min * 12 per min)
                while (dataset.data.length > 360) {
                    dataset.data.shift();
                }
            } else {
                // new sensor has appeared - add as new dataset
                let newChartData = getChartData({label: newData});
                chart.data.datasets.push(...newChartData.datasets);
            }

            // set latest temp on board status chart
            if (window.boardChart) {
                let boardDatasets = window.boardChart.data.datasets;
                let boardDataset = boardDatasets.find(ds => ds.label === label);
                if (boardDataset) {
                    if (newData.length) {
                        boardDataset.data[0]['temp'] = newData[newData.length - 1][1];
                        boardDataset.hidden = false;
                    } else {
                        boardDataset.data[0]['temp'] = null;
                        boardDataset.hidden = true;
                    }
                    window.boardChart.update();
                }
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
                                displayFormats: {
                                    second: 'HH:mm:ss',
                                    minute: 'HH:mm:ss',
                                },
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
                            radius: 1,
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
