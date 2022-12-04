const ctx = document.getElementById('temperature-chart');

function getChartData(data) {
    let sensor_ids = Object.keys(data)
        .filter(id => data[id].length); // exclude empty sensors
    return {
        datasets: sensor_ids.map(id => {
            return {
                label: id,
                data: data[id],
                borderWidth: 1
            }
        })
    };
}

function updateChartData(chart, newData) {
    chart.data.datasets.forEach(dataset => {
        if (dataset.label in newData && newData[dataset.label].length) {
            dataset.data.shift();
            dataset.data.push(...newData[dataset.label]);
        }
    });
    chart.options.scales.x.min = new Date(new Date().getTime() - 1.8e6); // 30 min
    chart.options.scales.x.max = new Date();
    chart.update();
}

fetch('/api/log_data')
    .then(response => response.json())
    .then(data => {
        const chart = new Chart(ctx, {
            type: 'line',
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
                        min: new Date(new Date().getTime() - 1.8e6), // 30 min
                        max: new Date(),
                    },
                    y: {
                        beginAtZero: true,
                        suggestedMax: 80.0,
                        title: {
                            display: true,
                            text: 'Temperature (°C)',
                        },
                    }
                },
                elements: {
                    point: {
                        radius: 0,
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
                })
        }, 5000);
    });

