const send = () => {
    Symphony.send({
        ServerMessage: {
            msg_type: "ShowPopup",
            state_id: 0,
            popup_id: "...",
            content: "...",
            title: "...",
        }
    })
}

Symphony.listenTo("listDir").then(() => send())

send()



