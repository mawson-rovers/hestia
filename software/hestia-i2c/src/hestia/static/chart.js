(function () { // prevent leaking into global scope
    const ctx = document.getElementById('temperature-chart');
    let durationMins = location.hash && location.hash.match(/d\d+/) ?
        location.hash.match(/d(\d+)/)[1] : 30; // default to 30 mins

    const colorPalette = [
        "hsl(211, 36%, 47%)",
        "hsl(204, 52%, 75%)",
        "hsl(30, 82%, 55%)",
        "hsl(30, 87%, 72%)",
        "hsl(113, 34%, 46%)",
        "hsl(109, 43%, 64%)",
        "hsl(47, 58%, 44%)",
        "hsl(45, 75%, 65%)",
        "hsl(177, 35%, 43%)",
        "hsl(172, 27%, 61%)",
        "hsl(359, 64%, 60%)",
        "hsl(1, 86%, 78%)",
        "hsl(12, 4%, 44%)",
        "hsl(18, 8%, 68%)",
        "hsl(338, 47%, 62%)",
        "hsl(341, 66%, 84%)",
        "hsl(315, 23%, 57%)",
        "hsl(315, 31%, 72%)",
        "hsl(21, 23%, 48%)",
        "hsl(18, 33%, 72%)",
    ];
    const heaterColors = {
        borderColor: 'hsl(347, 100%, 69%)',
        backgroundColor: 'hsla(347, 100%, 69%, 0.5)',
    };
    const savedColors = {};

    window.colorsForSensor = function (sensor_id) {
        if (sensor_id === 'heater') {
            return heaterColors;
        }
        if (!savedColors[sensor_id]) {
            const counter = Object.keys(savedColors).length;
            const color = colorPalette[counter % colorPalette.length];
            savedColors[sensor_id] = {
                borderColor: color,
                backgroundColor: color.replace(/^hsl/, "hsla").replace(/\)$/, ", 0.5)"),
            };
        }
        return savedColors[sensor_id];
    };

    // convert our timestamps to ISO8601 format to make Luxon happy
    const adaptTimestamps = seriesData =>
        seriesData.map(([timestamp, value]) => [timestamp.replace(' ', 'T'), value]);

    function getChartData(data) {
        let sensor_ids = Object.keys(data)
            .filter(id => data[id].length); // exclude empty sensors
        return {
            datasets: sensor_ids.map(function (id) {
                const colors = colorsForSensor(id);
                return {
                    label: id,
                    data: adaptTimestamps(data[id]),
                    borderWidth: 1,
                    borderColor: colors.borderColor,
                    backgroundColor: colors.backgroundColor,
                    fill: id === 'heater',
                    xAxisID: 'x',
                    yAxisID: id === 'heater' ? 'y2' : 'y1',
                };
            }),
        };
    }

    function minsToMillis(minutes) {
        return minutes * 60000;
    }

    function updateChartDuration(chart) {
        const now = new Date();
        const durationMillis = minsToMillis(durationMins);
        chart.options.scales.x.min = new Date(now.getTime() - durationMillis);
        chart.options.scales.x.max = now;
    }

    function updateChartData(chart, newDatasets) {
        Object.keys(newDatasets).forEach(function (label) {
            let newData = adaptTimestamps(newDatasets[label]);

            let dataset = chart.data.datasets.find(ds => ds.label === label);
            if (dataset) {
                // add all new data items (API can return multiple points)
                dataset.data.push(...newData);

                // limit samples to maximum visible (30 min * 12 per min)
                while (dataset.data.length > 1500) {
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
                            min: new Date(new Date().getTime() - minsToMillis(durationMins)),
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

            // load new data at regular intervals
            window.setInterval(() => {
                fetch('/api/data')
                    .then((response) => response.json())
                    .then((newData) => {
                        updateChartData(chart, newData);
                        updateChartDuration(chart);
                        chart.update();
                    });
            }, 5000);

            // click handlers for chart duration
            document.querySelectorAll(".duration-selector a").forEach(function (el) {
                let newDurationMins = el.getAttribute("data-duration-mins")
                el.addEventListener('click', (ev) => {
                    durationMins = newDurationMins;
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
