use gloo::console;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{
  Blob, DragEvent, Event, FileList, FormData, HtmlInputElement, MouseEvent, ProgressEvent,
  XmlHttpRequest,
};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};

pub enum MessageApp {
  StartUpload,
  UploadCompleted,
  FileSelected(String, web_sys::Blob),
  Progress(f64),
  Error(String),
}

pub struct App {
  file: Option<(String, web_sys::Blob)>,
  progress: f64,
  request: Option<XmlHttpRequest>,
}

impl Component for App {
  type Message = MessageApp;
  type Properties = ();

  fn create(_ctx: &Context<Self>) -> Self {
    Self {
      file: Option::default(),
      progress: 0f64,
      request: None,
    }
  }

  fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    match msg {
      MessageApp::StartUpload => {
        self.upload_file(ctx);
        true
      }
      MessageApp::FileSelected(name, file) => {
        self.file = Some((name, file));
        true
      }
      MessageApp::Progress(progress) => {
        self.progress = progress;
        true
      }
      MessageApp::UploadCompleted => true,
      MessageApp::Error(_) => true,
    }
  }

  fn view(&self, ctx: &Context<Self>) -> Html {
    html! {
        <div id="wrapper">
            <p id="title">{ "PasteFile" }</p>
            <label for="file-upload">
                <div
                    id="drop-container"
                    ondrop={ctx.link().callback(|event: DragEvent| {
                        event.prevent_default();
                        let files = event.data_transfer().unwrap().files();
                        Self::select_file(files)
                    })}
                    ondragover={Callback::from(|event: DragEvent| {
                        event.prevent_default();
                    })}
                    ondragenter={Callback::from(|event: DragEvent| {
                        event.prevent_default();
                    })}
                >
                    <i class="fa fa-cloud-upload"></i>
                    <p>{"Drop your images here or click to select"}</p>
                </div>
            </label>
            <input
                id="file-upload"
                type="file"
                accept="image/*,video/*,.doc,.xml,.txt"
                multiple={false}
                onchange={ctx.link().callback(move |e: Event| {
                    let input: HtmlInputElement = e.target_unchecked_into();
                    Self::select_file(input.files())
                })}
            />
            <div id = "previce-area">
            <button
            onclick={ctx.link().callback(move |_e: MouseEvent| {
              console::log!("*** Start Upload ***");
              MessageApp::StartUpload
          })}
            >{"Upload"}</button>
            </div>
            <div id="preview-area">
                { self.view_progress_bar() }
            </div>
        </div>
    }
  }
}

impl App {
  fn view_progress_bar(&self) -> Html {
    if let Some(_) = self.file {
      html! {
          <div>
              <progress value={self.progress.to_string()}></progress>
              <span>{ format!("{:.1}%", self.progress) }</span>
          </div>
      }
    } else {
      html! {}
    }
  }

  fn select_file(files: Option<FileList>) -> MessageApp {
    js_sys::try_iter(&files.unwrap())
      .unwrap()
      .unwrap()
      .next()
      .map(|v| {
        let v = v.unwrap();
        let b = Blob::from(v.clone());
        let f = web_sys::File::from(v);
        (f.name(), b)
      })
      .map(|(name, file)| MessageApp::FileSelected(name, file))
      .unwrap()
  }

  fn upload_file(&mut self, ctx: &Context<Self>) {
    let Some((filename, file)) = &self.file else {
      ctx
        .link()
        .send_message(MessageApp::Error("upload failed.".to_string()));
      return;
    };

    let f = FormData::new().unwrap();
    f.append_with_blob_and_filename("file", file, filename)
      .unwrap();
    let req = XmlHttpRequest::new().unwrap();
    req.open("POST", "http://127.0.0.1:8080/upload").unwrap();

    let link = ctx.link().clone();
    let progress: Closure<dyn Fn(ProgressEvent)> =
      Closure::new(move |e: ProgressEvent| link.send_message(MessageApp::Progress(e.loaded())));
    req
      .add_event_listener_with_callback("progress", progress.as_ref().unchecked_ref())
      .unwrap();

    let link = ctx.link().clone();
    let onloadend: Closure<dyn Fn(ProgressEvent)> =
      Closure::new(move |_e: ProgressEvent| link.send_message(MessageApp::UploadCompleted));
    req
      .add_event_listener_with_callback("onloadend", onloadend.as_ref().unchecked_ref())
      .unwrap();

    req.send_with_opt_form_data(Some(&f)).unwrap();
    self.request = Some(req);
  }
}
