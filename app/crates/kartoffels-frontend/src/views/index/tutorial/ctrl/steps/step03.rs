use super::prelude::*;

const CMD: &str = "git clone https://github.com/patryk27/kartoffel";

static MSG: LazyLock<Msg<&'static str>> = LazyLock::new(|| Msg {
    title: Some(" tutorial (3/16) "),

    body: vec![
        MsgLine::new("look at you, learning so fast - *NEXT LESSON!*"),
        MsgLine::new(""),
        MsgLine::new("run this:"),
        MsgLine::new(format!("    {CMD}")),
        MsgLine::new(""),
        MsgLine::new("... and press [`enter`] once you're ready"),
    ],

    buttons: vec![
        MsgButton::new("copy-command", KeyCode::Char('c'), "copy"),
        MsgButton::confirm("next", "next"),
    ],
});

pub async fn run(ctxt: &mut TutorialCtxt) -> Result<()> {
    loop {
        match ctxt.game.msg(&MSG).await? {
            "copy" => {
                ctxt.game.copy(CMD).await?;
            }

            "next" => {
                break;
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}
