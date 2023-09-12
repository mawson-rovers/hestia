(function () { // prevent leakage into global scope
    const boards = ["top", "bottom"];
    const host = document.body.dataset.fetchHost || "";
    const temp = getFieldElements('temp');
    const heaterPower = getFieldElements('heater-power');
    const heaterMode = getFieldElements('heater-mode');
    const heaterDuty = getFieldElements('heater-duty');
    const targetTemp = getFieldElements('target-temp');
    const targetSensor = getFieldElements('target-sensor');
    const boardChartElement = getFieldElements('board-chart');

    function getFieldElements(id) {
        return Object.fromEntries(
            boards.map(board => [board, document.getElementById(`${id}-${board}`)])
        );
    }

    function updateStatus(status_data) {
        boards.forEach(board => {
            let data = status_data[board] || {};
            temp[board].value = data["target_sensor_temp"] ?? "n/a";
            heaterPower[board].value = data["heater_power"] ?? "n/a";
            heaterMode[board].value = data['heater_mode'] ?? "n/a";
            heaterDuty[board].value = data["heater_duty"] ?? "n/a";
            targetTemp[board].value = data["target_temp"] ?? "n/a";
            targetSensor[board].value = data["target_sensor"] ?? "n/a";

            window.boardChart ??= {};
            if (!window.boardChart[board] && data['sensor_info']) {
                window.boardChart[board] = newBoardChart(
                    board, boardChartElement[board], data['sensor_info']);
            }
        });
    }

    function newBoardChart(board, ctx, data) {
        let mounted = 0;
        let datasets = Object.keys(data)
            .filter(id => !/^Circuit/.test(data[id]['label'])) // exclude non-temp data
            .map(id => {
                let x = Math.abs(data[id]['pos_x']);
                let y = Math.abs(data[id]['pos_y']);
                let isMounted = (data[id]['label'] === "Mounted");
                if (isMounted) {
                    mounted += 1;
                    x = 96;
                    y = 90 - mounted * 10;
                }
                return {
                    label: `${board}/${id}`,
                    data: [{ x: x, y: y, temp: null }],
                    unit: data[id]['unit'],
                    borderWidth: -20,  // get rid of the space left by the axes
                    backgroundColor: 'hsla(0, 0%, 100%, 0.2)',
                    pointRadius: isMounted ? 8.0 : 16.0,
                    hoverPointRadius: isMounted ? 12.0 : 20.0,
                };
            });

        const topImage = new Image();
        topImage.src = '/static/hestia-board-top.png';
        const bottomImage = new Image();
        bottomImage.src = '/static/hestia-board-bottom-flipped.png';

        const plugin = {
            id: 'boardBackground',
            beforeDraw: (chart) => {
                let image = chart.options.flipped ? bottomImage : topImage;
                if (image.complete) {
                    const ctx = chart.ctx;
                    const { top, left, bottom, right } = chart.chartArea;
                    ctx.drawImage(image, left, top, left + (right - left) * 0.8, bottom);
                } else {
                    image.onload = () => chart.draw();
                }
            }
        };

        return new Chart(ctx, {
            type: 'scatter',
            data: {
                datasets: datasets,
            },
            plugins: [plugin],
            options: {
                aspectRatio: 1.2,
                flipped: false,
                scales: {
                    // scales are adjusted so plot aligns with background image
                    x: {
                        min: 0.0,
                        max: 112.0, // x > 92 used for mounted sensors
                        display: false,
                    },
                    y: {
                        min: -2.0,
                        max: 96.0,
                        display: false,
                    }
                },
                elements: {
                    point: {
                        radius: 16.0,
                        hoverRadius: 20.0,
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
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'heater_mode': mode, 'board': board });
        });
    });

    // heater duty buttons
    document.querySelectorAll(".set-heater-duty").forEach(el => {
        let board = el.getAttribute("data-board");
        let values = document.getElementById(`heater-duty-${board}-values`);
        el.addEventListener('click', () => {
            let duty = Math.round(Number(values.value) * 255);
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'heater_duty': duty, 'board': board });
        });
    });

    // target temperature buttons
    document.querySelectorAll(".set-target-temp").forEach(el => {
        let board = el.getAttribute("data-board");
        let values = document.getElementById(`target-temp-${board}-values`);
        el.addEventListener('click', () => {
            let temp = Number(values.value);
            postStatusUpdate({ 'target_temp': temp, 'board': board });
        });
    });

    // target sensor buttons
    document.querySelectorAll(".set-target-sensor").forEach(el => {
        el.addEventListener('click', () => {
            let sensor = el.getAttribute('data-sensor');
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'target_sensor': sensor, 'board': board });
        });
    });

    // flip board links
    document.querySelectorAll(".board-flip").forEach(el => {
        let board = el.getAttribute("data-board");
        el.addEventListener('click', (ev) => {
            if (board in window.boardChart) {
                let options = window.boardChart[board].options;
                options.flipped = !options.flipped;
                window.boardChart[board].update('none');
            }
            ev.preventDefault();
            ev.stopPropagation();
        });
    });

    if (host) {
        document.querySelectorAll(".api-links a").forEach(link => {
            link.setAttribute("href", host + link.getAttribute("href"));
        });
    }
})();
