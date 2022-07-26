function get_invite_key(){
    let hash = window.location.hash;

    let params = new URLSearchParams(hash.substring(1));

    let invite_key = params.get("invite_key");

    return invite_key;
}

async function load_invite() {
    let invite_key = get_invite_key();

    if (invite_key == null) {
        return null;
    }
    try {
        let request = window.fetch(`/api/invite/${invite_key}`);

        let response = await request;

        if (response.status != 200) {
            return null;
        }

        let invite = response.json();

        /*
            pub uid: i32,
            pub email: String,
            pub names: String,
            pub guests: Vec<DbGuest>,
        */

        return invite;

    } catch (e) {
        // TODO:
        throw e;
    }

}

async function save_rsvp(guests) {
    let rsvp = {
        guests: guests
    };
    let invite_key = get_invite_key();

    let request = new Request(`/api/rsvp/${invite_key}`, {
        body: JSON.stringify(rsvp),
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json'
        }
    });

    let response = await fetch(request);

    return response.status == 200;
}

async function open_invite(email, password) {
    let invite = {
        email: email,
        password: password
    };

    let request = new Request(`/api/invite/lookup`, {
        body: JSON.stringify(invite),
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        }
    });

    let response = await fetch(request);

    if (response.status == 200) {
        return await response.json()
    } 

    console.log("unable to open invite", response);

    return {};
}

function delay(time) {
    return new Promise(resolve => setTimeout(resolve, time));
}

async function tick(type) {
    let tick_el = document.getElementById("tick");

    tick_el.style.top = "50%";
    tick_el.className = type;

    await delay(3000);
    
    tick_el.style.top = "-50%";

    await delay(1000);

    tick_el.className = "";
}

function create_guest(guest) {
    let template = document.getElementById("rsvp-template");
    let guestEl = template.cloneNode(true);
    let coming = guestEl.querySelector(".coming");
    let not_coming = guestEl.querySelector(".not-coming");
    let guest_name = guestEl.querySelector(".name");

    coming.name = "rsvp-" + guest.guestid;
    not_coming.name = "rsvp-" + guest.guestid;

    let update = () => {
    	guest_name.parentElement.className = coming.checked ? "title attending" : "title";
    };

    coming.addEventListener("change", () => {
        update();
    }); 
    not_coming.addEventListener("change", () => {
        update();
    }); 

    guestEl.style.display = "block";
    guestEl.guestid = guest.guestid;
    guest_name.innerText = guest.name;
    coming.checked = guest.attending == "Attending";
    not_coming.checked = guest.attending == "NotAttending";
    update();
    Array.from(guestEl.querySelectorAll("input.diet")).forEach(el => el.checked = guest.diet_options.indexOf(el.dataset.value) >= 0);

    console.log(guest);
    return guestEl;
}

function decode_guest(guest_el) {
    return {
        guestid: guest_el.guestid,
        attending: guest_el.querySelector(".coming").checked 
	               ? "Attending" 
	               : (guest_el.querySelector(".not-coming").checked
			       ? "NotAttending"
			       : "NotResponded"
		         ),
        diet_options: Array.from(guest_el.querySelectorAll("input.diet")).filter(el=>el.checked).map(el=>el.dataset.value)
    };
}

function setup_invite(invite) {
    if (invite == null) {
        let invite_not_found = document.getElementById("invite-not-found");
        invite_not_found.style.display = "block";
        return;
    }


    Array.from(document.querySelectorAll(".invite")).forEach(el => el.style.display="block");

    let name = invite.names;
    let invite_name = document.getElementById("invite-name");
    invite_name.innerHTML = name;

    let invite_saved = document.getElementById("invite-saved");
    invite.guests.map(guest => guest.attending != "NotResponded").filter(x => x).forEach(() =>{
    	invite_saved.style.display = "block";
    });

    let guest_elements = invite.guests.map(guest=> {
        let guest_el = create_guest(guest);

        let guest_list = document.getElementById("guest-list");

        guest_list.appendChild(guest_el);

        return guest_el;
    });

    document.getElementById("send").addEventListener("mouseup", async (e) => {
        e.preventDefault();
        e.stopPropagation();

        let guests = guest_elements.map(decode_guest);

        let result = await save_rsvp(guests);

        if (!result) {
            console.error("Error submitting data", guests);
            tick("bad");
            return;
        }

        tick("good");


	let invite_saved = document.getElementById("invite-saved");
	invite_saved.style.display = "block";
    });
}

function setup_open() {
    let invite_email = document.getElementById("invite-email");
    let invite_password = document.getElementById("invite-password");
    let invite_open = document.getElementById("open");
    let invite_unknown = document.getElementById("invite-unknown");

    let process_open = async () => {
        let invite = await open_invite(invite_email.value, invite_password.value);

        if (invite.invite_key) {
            tick("good");
            await(delay(1000));
            let params = new URLSearchParams();
            params.append("invite_key", invite.invite_key);
            window.location.hash = params.toString();
            // on_hashchange will reload the page now
            return;
        }

        tick("bad");

        invite_unknown.style.display = "block";
    };

    let trigger_open_keyboard = (e) => {
        if (e.code === "Enter") {
            e.preventDefault();
            e.stopPropagation();
        
            process_open();
        }
    };

    // When users press enter they expect things to happen
    invite_email.addEventListener("keyup", trigger_open_keyboard);
    invite_password.addEventListener("keyup", trigger_open_keyboard);
    invite_open.addEventListener("click", process_open);
}

async function on_load() {
    let invite = await load_invite(); 

    if (invite == null) {
        console.log("Invite not found");
        document.getElementById("invite-not-found").style.display="block";
        setup_open();
    }

    if (invite != null) {
        console.log("invite", invite);
        setup_invite(invite);
    }
}

function on_hashchange() {
    window.location.reload();
}

on_load();
window.addEventListener("hashchange", () => on_hashchange());
