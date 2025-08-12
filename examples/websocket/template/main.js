export function connect_input_and_button_to(ws_url, input_id, button_id) {
    let ws = null;

    const input  = document.getElementById(input_id);            
    input.spellcheck = false;
    input.disabled   = true;

    const button = document.getElementById(button_id);
    button.textContent = "connect";

    button.addEventListener(
        "click", (e) => {
            if (button.textContent == "connect") {
                ws = new WebSocket(ws_url);
                ws.addEventListener("open", (e) => {
                    console.log(e);
                    ws.send("test");
                });
                ws.addEventListener("message", (e) => {
                    console.log("ws got message: ", e.data);
                });
                ws.addEventListener("close", (e) => {
                    console.log("close:", e);

                    input.value = "";
                    input.disabled = true;

                    button.textContent = "connect";
                });

                input.disabled = false;

                button.textContent = "send";
            } else {   
                console.log("sending:", input.value);
                ws.send(input.value);
            }
        }
    );
}
