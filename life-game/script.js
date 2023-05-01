const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
let canvasWidth  = canvas.width;
let canvasHeight = canvas.height;

function drawGrid(x, y, w, h, size, lineWidth, color) {
  ctx.lineWidth = lineWidth;
  ctx.strokeStyle = color;

  const xEnd = x + size * w;
  const yEnd = y + size * h;
  
  ctx.beginPath();
  for(let j=0; j<=w; ++j){
    const xTemp = x + j * size;
    ctx.moveTo(xTemp, y);
    ctx.lineTo(xTemp, yEnd);
  }
  for(let i=0; i<=h; ++i){
    const yTemp = y + i * size;
    ctx.moveTo(x, yTemp);
    ctx.lineTo(xEnd, yTemp);
  }
  ctx.stroke();
}

class CellularAutomaton {
  constructor(x, y, w, h, size, lineWidth, gridColor, colorDead, colorLive, cells) {
    this.x = x;
    this.y = y;
    this.w = w;
    this.h = h;
    this.size = size;
    this.lineWidth = lineWidth;
    this.gridColor = gridColor;
    this.colorDead = colorDead;
    this.colorLive = colorLive;
    this.cells = cells;
  }

  state(i, j) {
    i = (i + this.w) % this.w;
    j = (j + this.h) % this.h;
    return this.cells[j * this.w + i];
  }

  setState(i, j, value) {
    i = (i + this.w) % this.w;
    j = (j + this.h) % this.h;
    if(0<=i && i<this.w && 0<=j && j<this.h)
      this.cells[j * this.w + i] = value;
  }

  update() {
    const nextWorld = [];
    for(let i=0; i<this.h; ++i) {
      for(let j=0; j<this.w; ++j) {
        let nextState = 0;
        let live = 
          this.state(j-1, i-1) + this.state(j, i-1) + this.state(j+1, i-1)
        + this.state(j-1, i)                        + this.state(j+1, i)
        + this.state(j-1, i+1) + this.state(j, i+1) + this.state(j+1, i+1);
        if(this.state(j, i)) {
          if(live < 2){
            nextState = 0;
          } else if(live < 4) {
            nextState = 1;
          } else {
            nextState = 0;
          }
        } else {
          if(live == 3) {
            nextState = 1;
          } else {
            nextState = 0;
          }
        }
        nextWorld.push(nextState);
      }
    }
    this.cells = nextWorld;
  }

  drawCells() {
    for(let i=0; i<this.h; ++i) {
      for(let j=0; j<this.w; ++j) {
        if(this.cells[i*this.w+j]) {
          ctx.fillStyle = this.colorLive;
        } else {
          ctx.fillStyle = this.colorDead;
        }
        ctx.fillRect(this.x + j*this.size, this.y + i*this.size, this.size, this.size);
      }
    }
  }

  draw() {
    this.drawCells();
    drawGrid(this.x, this.y, this.w, this.h, this.size, this.lineWidth, this.gridColor);
  }
}

let col = 36;
let row = 26;
let cells = [
//0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 1
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 2
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 3
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 4
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 5
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 6
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 7
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //10
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //11
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //12
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //13
  0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //14
  0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //15
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //16
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //17
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //18
  0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //19
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //20
  0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //21
  0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //22
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //23
  0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //24
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //25
];
let cellSize = 20;
let gridLeft = (canvasWidth - cellSize * col) / 2;
let gridTop = (canvasHeight - cellSize * row) / 2;
let ca = new CellularAutomaton(gridLeft, gridTop, col, row, cellSize, 1, 'gray', 'black', '#0f0', cells);


let buttonRecord = document.getElementById("record");
let buttonSave = document.getElementById("save");
let buttonCancel = document.getElementById("cancel");
let zip;

function record() {
  zip = new JSZip();
  recording = true;
  buttonRecord.disabled = true;
  buttonRecord.innerText = "Recording...";
  buttonSave.disabled = false;
  buttonCancel.disabled = false;
}

function save() {
  zip.generateAsync({type: "blob"}).then(function(blob){
    saveAs(blob, "images.zip");
  });
  zip = null;
  recording = false;
  buttonRecord.innerText = "Record";
  buttonRecord.disabled = false;
  buttonSave.disabled = true;
  buttonCancel.disabled = true;
}

function cancel() {
  zip = null;
  recording = false;
  buttonRecord.innerText = "Record";
  buttonRecord.disabled = false;
  buttonSave.disabled = true;
  buttonCancel.disabled = true;
}

let running = true;
let recording = false;
let count = 0;
let frameCount = 0;
const defaultFpms = 0.0015;
let fpms = defaultFpms;
let mspf = 1 / fpms;

let deltaOrigin;
let delta = 0;

function stop() {
  delta += performance.now() - deltaOrigin;
}
function start() {
  deltaOrigin = performance.now();
}

function requestUpdate(timestamp) {
  let elapsed = delta + timestamp - deltaOrigin;
  let numUpdate = Math.floor(elapsed * fpms);
  if(0 < numUpdate) {
    deltaOrigin = timestamp;
    delta = elapsed - mspf * numUpdate;
  }
  for (let i=0; i<numUpdate; ++i) {
    /* update */
    ca.update();
  
    /* draw */
    ctx.fillStyle = 'rgba(0,0,0,1)';
    ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    ca.draw();
    
    ++frameCount;
    /* save */
    if(recording) {
      canvas.toBlob((blob) => {
        zip.file("frame" + frameCount + ".png", blob, {binary: true});
      });
    }

  }
  if(running) {
    window.requestAnimationFrame(requestUpdate);
  }
}

ca.draw();
start();
window.requestAnimationFrame(requestUpdate);


const buttonPlay = document.getElementById("button-play");

function togglePlayPause() {
  if(running) {
    stop();
    running = false;
    buttonPlay.innerText = "Play";
  } else {
    start();
    running = true;
    buttonPlay.innerText = "Pause";
    requestUpdate();
  }
}

const sliderSpeed = document.getElementById("speed");
sliderSpeed.addEventListener("input", (event) => {
  let playSpeed = 2 ** Number(sliderSpeed.value);
  fpms = playSpeed * defaultFpms;
  mspf = 1 / fpms;
});

buttonPlay.addEventListener("click", togglePlayPause);

canvas.addEventListener('click', (event)=>{
  let j = Math.floor((event.offsetX - ca.x) / ca.size);
  let i = Math.floor((event.offsetY - ca.y) / ca.size);
  let value = ca.state(j, i);
  if(value) {
    value = 0;
  } else {
    value = 1;
  }
  ca.setState(j, i, value);
  ca.draw();
});

window.addEventListener('keydown', (event) => {
  if(event.code == 'Space') {
    event.preventDefault();
    togglePlayPause();
  }
  if(event.code == 'KeyZ') {
    let n = ca.w * ca.h;
    for(let i=0; i<n; ++i) {
      ca.cells[i] = 0;
    }
    ca.draw();
  }
  if(event.code == 'KeyC') {
    delta += mspf;
    if (!running)
      requestUpdate(deltaOrigin);
  }
});

