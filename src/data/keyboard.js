    
function circle_click(evt) {
  var circle = evt.target;
  var currentRadius = circle.getAttribute("r");
  if (currentRadius == 100)
    circle.setAttribute("r", currentRadius*2);
  else
    circle.setAttribute("r", currentRadius*0.5);
}

function layer_click(evt, layer) {
  alert("Layer " + layer);
}
