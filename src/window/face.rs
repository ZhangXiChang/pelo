use yew::prelude::*;

pub struct Face;
impl Component for Face {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!(<div class="Face">
            <div class="Button"><p>{"按钮"}</p></div>
        </div>)
    }
}
