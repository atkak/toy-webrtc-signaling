'use strict';

var username = null;

const webSocket = new WebSocket("ws://127.0.0.1:8080/ws");

webSocket.onmessage = async (msg) => {
  const { event, from, body } = JSON.parse(msg.data);

  switch (event) {
    case "joined":
      console.log(`${body.username} joined`);
      break;

    case "offer":
      console.log(`offer from ${from}: ${body}`);

      await pc.setRemoteDescription(body);
      await pc.setLocalDescription();
      await webSocket.send(JSON.stringify({ event: "answer", from: username, body: pc.localDescription }));
      break;

    case "answer":
      console.log(`answer from ${from}: ${body}`);

      await pc.setRemoteDescription(body);
      break;

    case "nomembers":
      console.log(`nomembers in the room. waiting for the joining.`);
      break;

    case "icecandidate":
      console.log(`icecandidate from ${from}: ${body}`);

      if (!body) {
        return;
      }

      const candidate = new RTCIceCandidate(body);
      await pc.addIceCandidate(candidate);
      break;
  }
};

const localVideo = document.getElementById('localVideo');
const remoteVideo = document.getElementById('remoteVideo');

const joinButton = document.getElementById('joinButton');
joinButton.onclick = async () => {
  username = document.getElementById('nameInput').textContent;

  try {
    await webSocket.send(JSON.stringify({ event: "join", from: username, body: { username: username } }));
    await addCameraMic();
  } catch (err) {
    console.error(err);
  }
};

const constraints = { audio: true, video: true };
const configuration = { iceServers: [{ urls: 'stun:stun.example.org' }] };
const pc = new RTCPeerConnection(configuration);

pc.onicecandidate = async ({ candidate }) => {
  await webSocket.send(JSON.stringify({ event: "icecandidate", from: username, body: candidate }));
}

pc.onnegotiationneeded = async () => {
  try {
    await pc.setLocalDescription();

    await webSocket.send(JSON.stringify({ event: "offer", from: username, body: pc.localDescription }));
  } catch (err) {
    console.error(err);
  }
};

pc.ontrack = ({ track, streams }) => {
  track.onunmute = () => {
    if (remoteVideo.srcObject) return;
    remoteVideo.srcObject = streams[0];
  };
};

async function addCameraMic() {
  try {
    const stream = await navigator.mediaDevices.getUserMedia(constraints);

    for (const track of stream.getTracks()) {
      pc.addTrack(track, stream);
    }

    localVideo.srcObject = stream;
  } catch (err) {
    console.error(err);
  }
}
