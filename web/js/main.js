'use strict';

const username = prompt("name");

const webSocket = new WebSocket("ws://localhost:8080/ws");

webSocket.onmessage = async (msg) => {
  const { event, from, body } = JSON.parse(msg.data);

  if (event == "joined") {
    console.log(`${body.username} joined`);
    if (!localVideo.srcObject) {
      await addCameraMic();
    }

    await pc.setLocalDescription();
    webSocket.send(JSON.stringify({ event: "offer", from: username, body: pc.localDescription }));
  } else if (event == "offer") {
    console.log(`offer from ${from}: ${body}`);

    await pc.setRemoteDescription(body);
    if (!localVideo.srcObject) {
      await addCameraMic();
    }

    await pc.setLocalDescription();
    webSocket.send(JSON.stringify({ event: "answer", from: username, body: pc.localDescription }));
  } else if (event == "answer") {
    console.log(`answer from ${from}: ${body}`);

    await pc.setRemoteDescription(body);
  }
};

const localVideo = document.getElementById('localVideo');
const remoteVideo = document.getElementById('remoteVideo');

const joinButton = document.getElementById('joinButton');
joinButton.onclick = async () => {
  try {
    webSocket.send(JSON.stringify({ event: "join", from: username, body: { username: username } }));
  } catch (err) {
    console.error(err);
  }
};

const constraints = { audio: true, video: true };
const configuration = { iceServers: [{ urls: 'stun:stun.example.org' }] };
const pc = new RTCPeerConnection(configuration);

pc.onicecandidate = ({ candidate }) => webSocket.send(JSON.stringify({ candidate }));

pc.onnegotiationneeded = async () => {
  try {
    await pc.setLocalDescription();

    signaling.send(JSON.stringify({ description: pc.localDescription }));
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
