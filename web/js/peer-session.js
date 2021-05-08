'use strict';

export default class PeerSession {
  constructor(username, signalingUrl) {
    this.username = username;
    this.onvideounmute = null;
    this.onclose = null;

    this.webSocket = new WebSocket(signalingUrl);
    this.webSocket.onopen = this.handleOnopen.bind(this);
    this.webSocket.onmessage = this.handleOnmessage.bind(this);
    this.webSocket.onerror = this.handleOnerror.bind(this);

    const configuration = { iceServers: [{ urls: 'stun:stun.example.org' }] };
    this.pc = new RTCPeerConnection(configuration);
    this.pc.onicecandidate = this.handleOnicecandidate.bind(this);
    this.pc.onnegotiationneeded = this.handleOnnegotiationneeded.bind(this);
    this.pc.ontrack = this.handleOntrack.bind(this);
  }

  close() {
    this.webSocket.send(JSON.stringify({ event: "leave", from: this.username }));
  }

  addMediaStream(stream) {
    for (const track of stream.getTracks()) {
      this.pc.addTrack(track, stream);
    }
  }

  async handleOnopen() {
    this.webSocket.send(JSON.stringify({ event: "join", from: this.username, body: { username: this.username } }));
  }

  async handleOnmessage(msg) {
    const { event, from, body } = JSON.parse(msg.data);

    switch (event) {
      case "joined":
        console.log(`${body.username} joined`);
        break;

      case "left":
        console.log(`${from} left`);

        this.pc.close();
        this.pc = null;
        this.webSocket.close();
        this.webSocket = null;

        this.onclose();
        break;

      case "offer":
        console.log(`offer from ${from}: ${body}`);

        await this.pc.setRemoteDescription(body);
        await this.pc.setLocalDescription();
        this.webSocket.send(JSON.stringify({ event: "answer", from: this.username, body: this.pc.localDescription }));
        break;

      case "answer":
        console.log(`answer from ${from}: ${body}`);

        this.pc.setRemoteDescription(body);
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
        this.pc.addIceCandidate(candidate);
        break;
    }
  }

  handleOnerror(error) {
    console.log(error);
  }

  async handleOnicecandidate({ candidate }) {
    this.webSocket.send(JSON.stringify({ event: "icecandidate", from: this.username, body: candidate }));
  }

  async handleOnnegotiationneeded() {
    try {
      await this.pc.setLocalDescription();

      this.webSocket.send(JSON.stringify({ event: "offer", from: this.username, body: this.pc.localDescription }));
    } catch (err) {
      console.error(err);
    }
  }

  handleOntrack({ track, streams }) {
    track.onunmute = () => {
      this.onvideounmute(streams[0]);
    };
  }
}
