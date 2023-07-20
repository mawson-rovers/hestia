(function () { // prevent leakage into global scope
    const host = document.body.dataset.fetchHost || "";
    const coreTemperature = document.getElementById('core-temperature');
    const heaterPower = document.getElementById('heater-power');
    const heaterMode = document.getElementById('heater-mode');
    const heaterDuty = document.getElementById('heater-duty');
    const targetTemp = document.getElementById('target-temp');
    const targetSensor = document.getElementById('target-sensor');
    const targetSensorLabel = document.getElementById('target-sensor-label');
    const boardChartElement = document.getElementById('board-chart');

    function updateStatus(data) {
        data = data["1"] || data["2"] || {};
        coreTemperature.value = data["target_sensor_temp"] ?? "n/a";
        heaterPower.value = data["heater_power"] ?? "n/a";
        heaterMode.value = data['heater_mode'] ?? "n/a";
        heaterDuty.value = data["heater_duty"] ?? "n/a";
        targetTemp.value = data["target_temp"] ?? "n/a";
        targetSensor.value = data["target_sensor"] ?? "n/a";
        targetSensorLabel.innerText = targetSensor.value;

        if (!window.boardChart && data['sensor_info']) {
            window.boardChart = newBoardChart(boardChartElement, data['sensor_info']);
        }
    }

    function newBoardChart(ctx, data) {
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
                        unit: data[id]['unit'],
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
                                    `${ctx.raw.temp} ${ctx.dataset.unit}` : "n/a";
                                return label;
                            },
                        },
                    },
                },
            },
        });
    }

    function fetchAndUpdateStatus() {
        fetch(host + '/api/status')
            .then(response => response.json())
            .then(data => updateStatus(data));
    }

    fetchAndUpdateStatus();
    setInterval(fetchAndUpdateStatus, 5000);

    function postStatusUpdate(data) {
        fetch(host + '/api/status', {
            method: 'POST',
            body: JSON.stringify({
                'board_id': 2,
                ...data
            }),
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
            postStatusUpdate({ 'heater_mode': mode });
        });
    });

    // heater duty buttons
    document.querySelectorAll(".set-heater-duty").forEach(el => {
        el.addEventListener('click', () => {
            let duty = Number(el.getAttribute('data-duty'));
            postStatusUpdate({ 'heater_duty': duty });
        });
    });

    document.querySelectorAll(".set-custom-heater-duty").forEach(el => {
        const customDuty = document.getElementById("custom-heater-duty");
        el.addEventListener('click', () => {
            let duty = Math.floor(Number(customDuty.value) * 255);
            postStatusUpdate({ 'heater_duty': duty });
        });
    });

    // target temperature buttons
    document.querySelectorAll(".set-target-temp").forEach(el => {
        el.addEventListener('click', () => {
            let temp = Number(el.getAttribute('data-temp'));
            postStatusUpdate({ 'target_temp': temp });
        });
    });

    // target sensor buttons
    document.querySelectorAll(".set-target-sensor").forEach(el => {
        el.addEventListener('click', () => {
            let sensor = el.getAttribute('data-sensor');
            postStatusUpdate({ 'target_sensor': sensor });
        });
    });

    // flip board link
    document.getElementById("board-flip")?.addEventListener('click', (ev) => {
        document.getElementById("board-chart-container").classList.toggle("flip");
        ev.preventDefault();
        ev.stopPropagation();
    });
})();
