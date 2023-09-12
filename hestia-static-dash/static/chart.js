(function () { // prevent leaking into global scope
    const host = document.body.dataset.fetchHost || "";
    const ctx = document.getElementById('temperature-chart');
    let durationMins = location.hash && location.hash.match(/d\d+/) ?
        location.hash.match(/d(\d+)/)[1] : 30; // default to 30 mins

    const series = [
        {
            label: "top/TH1",
            color: "hsl(190, 100%, 50%)",
        },
        {
            label: "top/TH3",
            color: "hsl(210, 100%, 50%)",
        },
        {
            label: "top/U4",
            color: "hsl(225, 67%, 50%)",
        },
        {
            label: "top/J7",
            color: "hsl(265, 100%, 40%)",
        },
        {
            label: "top/heater_power",
            color: "hsla(347, 100%, 54%, 0.5)",
            borderColor: "hsl(347, 100%, 54%)",
            fill: true,
            yAxisID: "y2",
        },
        {
            label: "bottom/TH1",
            color: "hsl(80, 100%, 50%)",
        },
        {
            label: "bottom/TH3",
            color: "hsl(110, 100%, 50%)",
        },
        {
            label: "bottom/U4",
            color: "hsl(125, 67%, 40%)",
        },
        {
            label: "bottom/J7",
            color: "hsl(165, 100%, 50%)",
        },
        {
            label: "bottom/heater_power",
            color: "hsla(36, 100%, 54%, 0.5)",
            borderColor: "hsl(36, 100%, 54%)",
            fill: true,
            yAxisID: "y2",
        },
    ];
    Chart.defaults.color = 'rgba(255, 255, 255, 0.9)';
    Chart.defaults.borderColor = 'rgba(255, 255, 255, 0.2)';

    function colorForTemp(temp) {
        let constrained = Math.max(Math.min(temp, 100), 0);
        let hue = 224 - Math.round(constrained / 100 * 224)
        return `hsla(${hue}, 100%, 50%, 0.4)`;
    }

    const borderColor = color => color.replace(/^hsl/, "hsla").replace(/\)$/, ", 0.5)");

    // convert our timestamps to ISO8601 format to make Luxon happy
    const adaptTimestamps = seriesData =>
        seriesData.map(([timestamp, value]) => [timestamp.replace(' ', 'T'), value]);

    function getChartData(payloadData) {
        return series.map(function (s) {
            let [board, id] = s.label.split("/");
            let data = board in payloadData && id in payloadData[board] ?
                payloadData[board][id] : [];
            return {
                label: s.label,
                data: adaptTimestamps(data),
                borderWidth: 1,
                borderColor: s.borderColor || borderColor(s.color),
                backgroundColor: s.color,
                fill: s.fill || false,
                xAxisID: 'x',
                yAxisID: s.yAxisID || "y1",
            };
        });
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

    function updateChartData(chart, board, newDatasets) {
        Object.keys(newDatasets).forEach(function (label) {
            let newData = adaptTimestamps(newDatasets[label]);
            prefix_label = `${board}/${label}`;

            let dataset = chart.data.datasets.find(ds => ds.label === prefix_label);
            if (dataset) {
                // add all new data items (API can return multiple points)
                dataset.data.push(...newData);

                // limit samples to maximum visible (120 min * 12 per min)
                while (dataset.data.length > 1500) {
                    dataset.data.shift();
                }
            } else {
                // Ignore other data
            }

            // set latest temp on board status chart
            if (window.boardChart) {
                Object.values(window.boardChart).forEach(boardChart => {
                    let boardDatasets = boardChart.data.datasets;
                    let boardDataset = boardDatasets.find(ds => ds.label === prefix_label);
                    if (boardDataset) {
                        if (newData.length) {
                            let temp = newData[newData.length - 1][1];
                            boardDataset.data[0]['temp'] = temp;
                            boardDataset.backgroundColor = colorForTemp(temp);
                            boardDataset.hidden = false;
                        } else {
                            boardDataset.data[0]['temp'] = null;
                            boardDataset.hidden = true;
                        }
                        boardChart.update();
                    }
                });
            }
        });
    }

    fetch(host + '/api/log_data')
        .then(response => {
            if (response.ok && response.headers.get("Content-Type").startsWith("application/json")) {
                return response.json();
            } else {
                throw response;
            }
        })
        .then(function (payloadData) {
            const chart = new Chart(ctx, {
                type: 'line',
                responsive: true,
                maintainAspectRatio: false,
                data: {
                    datasets: getChartData(payloadData),
                },
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
                            grid: {
                                display: false,
                            },
                            min: new Date(new Date().getTime() - minsToMillis(durationMins)),
                            max: new Date(),
                        },
                        y1: {
                            beginAtZero: true,
                            suggestedMax: 100.0,
                            title: {
                                display: true,
                                text: 'Temperature (°C)',
                            },
                        },
                        y2: {
                            beginAtZero: true,
                            suggestedMax: 10,
                            title: {
                                display: true,
                                text: 'Heater power (W)',
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
                            width: "200",
                        },
                    },
                },
            });

            // load new data at regular intervals
            window.setInterval(() => {
                fetch(host + '/api/data')
                    .then((response) => response.json())
                    .then((newData) => {
                        updateChartData(chart, "top", newData["top"] || {});
                        updateChartData(chart, "bottom", newData["bottom"] || {});
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
        })
        .catch((response) => {
            console.error("Failed to retrieve log_data:", response)
        });
})();
