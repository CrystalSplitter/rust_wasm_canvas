export class InputBindings {
    constructor(mouseX, mouseY) {
        this.mouseX = mouseX;
        this.mouseY = mouseY;
    }

    getMouseX() {
        return this.mouseX;
    }

    getMouseY() {
        return this.mouseY;
    }
}

// From: https://stackoverflow.com/a/17130415/6519699
export function  setMousePos(obj, canvas, evt) {
  var rect = canvas.getBoundingClientRect(), // abs. size of element
      scaleX = canvas.width / rect.width,    // relationship bitmap vs. element for X
      scaleY = canvas.height / rect.height;  // relationship bitmap vs. element for Y

  obj.mouseX = (evt.clientX - rect.left) * scaleX;
  obj.mouseY = (evt.clientY - rect.top) * scaleY;
}
