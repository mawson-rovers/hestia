(function () { // prevent leakage into global scope
    const coreTemperature = document.getElementById('core-temperature');
    const heaterMode = document.getElementById('heater-mode');
    const heaterDuty = document.getElementById('heater-duty');
    const targetTemp = document.getElementById('target-temp');
    const boardChartElement = document.getElementById('board-chart');

    function updateStatus(data) {
        coreTemperature.value = "center_temp" in data ? data["center_temp"] : "n/a";
        if ('heater_mode' in data) {
            heaterMode.value = data['heater_mode'];
        } else {
            heaterMode.value = "n/a";
        }
        heaterDuty.value = "heater_duty" in data ? data["heater_duty"] : "n/a";
        targetTemp.value = "target_temp" in data ? data["target_temp"] : "n/a";

        if (!window.boardChart) {
            window.boardChart = newBoardChart(boardChartElement, data['sensors']);
        }
    }

    function newBoardChart(ctx, data) {
        const tempFormat = new Intl.NumberFormat('en-US', {
            style: 'unit',
            unit: 'celsius',
            maximumSignificantDigits: 4
        });
        let mounted = 0;
        return new Chart(ctx, {
            type: 'scatter',
            data: {
                datasets: Object.keys(data).map(id => {
                    let x = Math.abs(data[id]['pos_x']);
                    let y = Math.abs(data[id]['pos_y']);
                    if (x === 0.0 && y === 0.0) {
                        mounted += 1;
                        x = 96;
                        y = 92 - mounted * 5;
                    }
                    let colors = window.colorsForSensor(id);
                    return {
                        label: id,
                        data: [{x: x, y: y, temp: null}],
                        borderWidth: 1,
                        borderColor: colors.borderColor,
                        backgroundColor: colors.backgroundColor,
                    };
                })
            },
            options: {
                aspectRatio: 1.2,
                scales: {
                    // scales are adjusted so plot aligns with background image
                    x: {
                        min: 3.0,
                        max: 113.0, // x > 92 used for mounted sensors
                        display: false,
                    },
                    y: {
                        min: 3.0,
                        max: 92.0,
                        display: false,
                    }
                },
                elements: {
                    point: {
                        radius: 8.0,
                        hoverRadius: 12.0,
                    },
                },
                plugins: {
                    legend: {
                        display: false,
                    },
                    tooltip: {
                        callbacks: {
                            label: function (ctx) {
                                let label = ctx.dataset.label || '';
                                if (label) label += ': ';
                                label += ctx.raw.temp !== null ?
                                    tempFormat.format(ctx.raw.temp) : "n/a";
                                return label;
                            },
                        },
                    },
                },
            },
        });
    }

    function fetchAndUpdateStatus() {
        fetch('/api/status')
            .then(response => response.json())
            .then(data => updateStatus(data));
    }

    fetchAndUpdateStatus();
    setInterval(fetchAndUpdateStatus, 5000);

    function postStatusUpdate(data) {
        fetch('/api/status', {
            method: 'POST',
            body: data,
            redirect: "follow",
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
        })
            .then(response => response.json())
            .then(data => updateStatus(data))
    }

    // heater mode buttons
    document.querySelectorAll(".set-heater-mode").forEach(el => {
        el.addEventListener('click', () => {
            let mode = el.getAttribute("data-mode");
            postStatusUpdate(JSON.stringify({
                'heater_mode': mode,
            }));
        });
    });

    // heater duty buttons
    document.querySelectorAll(".set-heater-duty").forEach(el => {
        el.addEventListener('click', () => {
            let duty = Number(el.getAttribute('data-duty'));
            postStatusUpdate(JSON.stringify({
                'heater_duty': duty,
            }));
        });
    });

    // target temperature buttons
    document.querySelectorAll(".set-target-temp").forEach(el => {
        el.addEventListener('click', () => {
            let temp = Number(el.getAttribute('data-temp'));
            postStatusUpdate(JSON.stringify({
                'target_temp': temp,
            }));
        });
    });

    // flip board link
    document.getElementById("board-flip")?.addEventListener('click', (ev) => {
        document.getElementById("board-chart-container").classList.toggle("flip");
        ev.preventDefault();
        ev.stopPropagation();
    });
})();
