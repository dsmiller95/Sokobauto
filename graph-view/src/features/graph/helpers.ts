

const minColor = [117/255, 70/255, 63/255, 255/255];
const maxColor = [7/255, 14/255, 227/255, 255/255];

export function getColorsByMaxTargets(maxOnTargets: number) {
  const colorsByTargets = Array(maxOnTargets + 1).fill(0).map((_, i) => {
    const t = maxOnTargets === 0 ? 0 : i / maxOnTargets;
    return minColor.map((minC, index) => {
      const maxC = maxColor[index];
      return minC * (1 - t) + maxC * t;
    });
  });
  return colorsByTargets;
}

export function colorToHex(color: number[]): string {
  if(color.length < 3) return '#000000';
  const r = Math.round(color[0] * 255);
  const g = Math.round(color[1] * 255);
  const b = Math.round(color[2] * 255);
  return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
}