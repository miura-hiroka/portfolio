const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const width = canvas.width = 600;
const height = canvas.height = 600;

let scale = 0.75;
let spread = Math.PI / 3;
let iter = 6;

function animate(timeStamp) {
  ctx.fillStyle = "#000";
  ctx.fillRect(0, 0, width, height);
  ctx.strokeStyle = "#6b6";
  ctx.lineWidth = 10;
  ctx.lineCap = "round";

  let len = height / 8;
  let angle = - Math.PI / 2;
  draw_shape(width/2, height * 7 / 8, len, angle, iter, 10);

  window.requestAnimationFrame(animate);
}

function draw_shape(x, y, len, angle, iter, lw) {
  if (iter <= 0) return;
  ctx.lineWidth = lw;
  ctx.beginPath();
  ctx.moveTo(x, y);
  let x2 = x + len * Math.cos(angle);
  let y2 = y + len * Math.sin(angle);
  ctx.lineTo(x2, y2);
  ctx.stroke();
  let new_len = len * scale;
  --iter;
  lw *= 0.9;
  draw_shape(x2, y2, new_len, angle + spread, iter, lw);
  draw_shape(x2, y2, new_len, angle - spread, iter, lw);
}

const input_scale = document.getElementById("scale");
const input_spread = document.getElementById("spread");
const input_iter = document.getElementById("iter");

input_scale.addEventListener("input", (event) => {
  scale = Number(input_scale.value);
});

input_spread.addEventListener("input", (event) => {
  spread = Number(input_spread.value) * Math.PI / 180;
});

input_iter.addEventListener("input", (event) => {
  iter = Number(input_iter.value);
});

window.requestAnimationFrame(animate);
