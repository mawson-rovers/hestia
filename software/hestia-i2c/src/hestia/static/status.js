(function () { // prevent leakage into global scope
    const coreTemperature = document.getElementById('core-temperature');
    const heaterMode = document.getElementById('heater-mode');
    const heaterPowerLevel = document.getElementById('heater-power-level');
    const heaterToggle = document.getElementById('heater-toggle');
    const boardChartElement = document.getElementById('board-chart');

    function updateStatus(data) {
        coreTemperature.value = "center_temp" in data ? data["center_temp"] : "n/a";
        if ('heater_enabled' in data) {
            heaterMode.value = data['heater_enabled'] ? "ON" : "OFF";
        } else {
            heaterMode.value = "n/a";
        }
        heaterPowerLevel.value = ("heater_pwm_freq" in data && data["heater_pwm_freq"]) ?
            data["heater_pwm_freq"] : "n/a";

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
                datasets: Object.keys(data).map(k => {
                    let x = Math.abs(data[k]['pos_x']);
                    let y = Math.abs(data[k]['pos_y']);
                    if (x === 0.0 && y === 0.0) {
                        mounted += 1;
                        x = 96;
                        y = 92 - mounted * 5;
                    }
                    return {
                        label: k,
                        data: [{x: x, y: y, temp: null}],
                        borderWidth: 1,
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

    // toggle heater button
    heaterToggle.addEventListener('click', (e) => {
        let newValue = heaterMode.value !== "ON";
        postStatusUpdate(JSON.stringify({
            'heater_enabled': newValue,
        }));
    });

    // power level buttons
    document.querySelectorAll(".set-heater-power").forEach(el => {
        el.addEventListener('click', () => {
            let powerLevel = Number(el.getAttribute('data-power-level'));
            postStatusUpdate(JSON.stringify({
                'heater_pwm_freq': powerLevel,
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
