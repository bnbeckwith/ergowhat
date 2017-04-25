function layer_click(layer) {
  alert("Layer " + layer);
}

function templayeron(layer) {
  document.getElementById("layer" + layer).setAttribute('visibility','visible')
}

function templayeroff(layer) {
  document.getElementById("layer" + layer).setAttribute('visibility','hidden')
}

window.onload = function(){
  document.getElementById("layer0").setAttribute('visibility','visible');  
}
