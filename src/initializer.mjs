export default function initializer() {
  return {
    onStart: () => {
      console.log("Loading...");
      console.time("trunk-initializer");

      const loadingEl = document.getElementById("loading-text");
      const canvasEl = document.getElementById("loading-canvas");
      const ctx = canvasEl.getContext("2d");
      let grid = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      ];
      const size = grid.length;
      const cellSize = canvasEl.width / size;

      let loading = 0;
      let dots = 0;
      function draw() {
        loading++;
        if (loading % 2 == 0) {
          dots = (dots + 1) % 4;
          loadingEl.innerText = `Loading${".".repeat(dots) + " ".repeat(3 - dots)}`;
        }
        ctx.fillStyle = "black";
        ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);

        ctx.fillStyle = "white";
        let next = grid.map((arr) => [...arr]);
        for (let y = 1; y < size - 1; y++) {
          for (let x = 1; x < size - 1; x++) {
            let neighbors = 0;
            for (let dy = -1; dy <= 1; dy++) {
              for (let dx = -1; dx <= 1; dx++) {
                if (dy === 0 && dx === 0) continue;
                const ny = y + dy;
                const nx = x + dx;
                neighbors += grid[ny][nx];
              }
            }
            if (neighbors == 2) next[y][x] = grid[y][x];
            else if (neighbors == 3) next[y][x] = 1;
            else next[y][x] = 0;

            if (grid[y][x])
              ctx.fillRect(
                x * cellSize + 0.5,
                y * cellSize + 0.5,
                cellSize + 0.5,
                cellSize + 0.5,
              );
          }
        }
        grid = next;
        setTimeout(() => requestAnimationFrame(draw), 125);
      }
      draw();
    },
    onProgress: ({ current, total }) => {
      if (total) {
        console.log("Loading...", Math.round((current / total) * 100), "%");
      } else {
        console.log("Loading...", current, "bytes");
      }
    },
    onComplete: () => {
      const loadingEl = document.getElementById("loading-container");
      if (loadingEl) loadingEl.remove();
      console.timeEnd("trunk-initializer");
    },
    onSuccess: (wasm) => {},
    onFailure: (error) => {},
  };
}
