use ncurses::*;
use ncurses::menu::*;
use mysql::*;
use mysql::prelude::*;

fn main () -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let url = "mysql://denis:12345678@localhost:3306/rust";
    let pool = Pool::new(url)?;
    
    let mut screen = Screen::init()?;

    // Initialize colors
    if screen.has_colors()
    {
        screen.start_color()?;
        screen.init_pair(1, COLOR_WHITE, COLOR_BLUE)?; // Header
        screen.init_pair(2, COLOR_BLACK, COLOR_CYAN)?; // Selected item
        screen.init_pair(3, COLOR_WHITE, COLOR_BLACK)?; // Normal item
        screen.init_pair(4, COLOR_YELLOW, COLOR_BLACK)?; // Status
    }

    screen.keypad(true);
    screen.noecho()?;
    screen.cbreak()?;

//    let max_y = screen.lines();
//    let max_x = screen.cols();

    // Create menu items
    let items = vec![
        MenuItem::new("Afficher tous", "Afficher tous"),
        MenuItem::new("Afficher un individu", "Afficher un individu"),
        MenuItem::new("Ajouter un individu", "Ajouter un individu"),
        MenuItem::new("Supprimer un individu", "Supprimer un individu"),
        MenuItem::new("NU1", "Pas encore utilisé"),    
        MenuItem::new("NU2", "Pas encore utilisé"),
        MenuItem::new("NU3", "Pas encore utilisé"),
        MenuItem::new("Help", "Show help information"),
        MenuItem::new("Reset", "Réinitialisation"),
        MenuItem::new("Exit", "Quit the application"),
    ];

    // Create menu
    let mut menu = Menu::new(items);
    menu.set_format(10, 1); // 10 rows, 1 column

    // Set attributes
    if screen.has_colors() {
        menu.set_fore(attr::color_pair(2) | attr::A_BOLD);
        menu.set_back(attr::color_pair(3));
    }

    // Create window for the menu
    let menu_height = 14;
    let menu_width = 60;
    let menu_y = 2;
    let menu_x = 2;

    let mut menu_win = screen.newwin(menu_height, menu_width, menu_y, menu_x)?;

    // Draw interface
    draw_title(&mut screen)?;
    draw_instructions(&mut screen)?;

    // Post menu
    menu.post()?;

    screen.refresh()?;

    // Main loop
    loop
    {
        // Draw the menu
        draw_menu(&mut screen, &mut menu_win, &menu)?;
        screen.wrefresh(&mut menu_win)?;

        // Draw status showing current selection
        draw_status(&mut screen, &menu)?;

        let ch = screen.getch()?;

        match ch 
        {
            // Arrow keys for navigation
            k if k == key::KEY_UP => 
            {
                menu.driver(REQ_PREV_ITEM)?;
            }
            k if k == key::KEY_DOWN => 
            {
                menu.driver(REQ_NEXT_ITEM)?;
            }

            // Home/End for first/last
            k if k == key::KEY_HOME => 
            {
                menu.driver(REQ_FIRST_ITEM)?;
            }
            k if k == key::KEY_END => 
            {
                menu.driver(REQ_LAST_ITEM)?;
            }

            // Page up/down (jump by 5)
            k if k == key::KEY_PPAGE => 
            {
                for _ in 0..5 
                {
                    menu.driver(REQ_PREV_ITEM)?;
                }
            }
            k if k == key::KEY_NPAGE => 
            {
                for _ in 0..5 
                {
                    menu.driver(REQ_NEXT_ITEM)?;
                }
            }

            // Enter to select
            k if k == 10 || k == 13 || k == key::KEY_ENTER => 
            {
                if let Some(item) = menu.current_item() 
                {
                    let item = item.borrow();
                    if item.name() == "Exit"
                    {
                        break;
                    }
                    // Exécution de la commande
                    execution ( &mut screen, &item, pool.clone() )?;
                }
            }

            // q to quit
            k if k == b'q' as i32 || k == b'Q' as i32 => 
            {
                break;
            }

            // Pass printable characters to pattern matching
            k if k >= 0x20 && k < 0x7f => 
            {
                menu.driver(k)?;
            }

            _ => {}
        }
    }

    // Unpost menu
    menu.unpost()?;

    Ok(())
}

fn draw_title(screen: &mut Screen) -> std::result::Result<(), Box<dyn std::error::Error>>
{
    screen.clear()?;

    if screen.has_colors() 
    {
        screen.attron(attr::color_pair(1))?;
    }
    screen.attron(attr::A_BOLD)?;

    let title = " ncurses-pure Menu Demo ";
//    let x = (screen.cols() - title.len() as i32) / 2;
    let x = 15;
    screen.mvaddstr(1, x, title)?;

    screen.attroff(attr::A_BOLD)?;
    if screen.has_colors() 
    {
        screen.attroff(attr::color_pair(1))?;
    }

    Ok(())
}

fn draw_instructions(screen: &mut Screen) -> std::result::Result<(), Box<dyn std::error::Error>>
{
    if screen.has_colors() {
        screen.attron(attr::color_pair(4))?;
    }

    screen.mvaddstr(
        37,
        2,
        "Navigation: UP/DOWN arrows, HOME/END, PGUP/PGDN",
    )?;
    screen.mvaddstr( 38, 2, "Select: ENTER | Type to search | Quit: 'q'")?;

    if screen.has_colors() {
        screen.attroff(attr::color_pair(4))?;
    }

    Ok(())
}

fn draw_menu(screen: &mut Screen, menu_win: &mut Window, menu: &menu::Menu) -> std::result::Result<(), Box<dyn std::error::Error>>
{
    menu_win.erase()?;

    // Draw border
    menu_win.box_(0, 0)?;

    // Draw title
    menu_win.attron(attr::A_BOLD)?;
    menu_win.mvaddstr(0, 2, " File Menu ")?;
    menu_win.attroff(attr::A_BOLD)?;

    // Draw items
    let current_idx = menu.current_item_index();
    for i in 0..menu.item_count() {
        let y = (i + 2) as i32;
        let is_current = i == current_idx;

        if let Some(item_rc) = menu.item(i) {
            let item = item_rc.borrow();

            if is_current {
                menu_win.attron(menu.fore())?;
                // Fill the line with background
                menu_win.mv(y, 1)?;
                for _ in 0..(menu_win.getmaxx() - 2) {
                    menu_win.addch(b' ' as ChType)?;
                }
            } else {
                menu_win.attron(menu.back())?;
            }

            // Draw item name
            menu_win.mvaddstr(y, 2, &format!(" {} ", item.name()))?;

            // Draw description on the right
            let desc = item.description();
            let desc_x = menu_win.getmaxx() - desc.len() as i32 - 3;
            if desc_x > item.name().len() as i32 + 5 {
                menu_win.mvaddstr(y, desc_x, desc)?;
            }

            if is_current {
                menu_win.attroff(menu.fore())?;
            } else {
                menu_win.attroff(menu.back())?;
            }
        }
    }

    // Show pattern if any
    let pattern = menu.pattern();
    if !pattern.is_empty() {
        menu_win.mvaddstr(menu_win.getmaxy() - 1, 2, &format!("Search: {}", pattern))?;
    }

    screen.wrefresh(menu_win)?;

    Ok(())
}

fn draw_status(screen: &mut Screen, menu: &menu::Menu) -> std::result::Result<(), Box<dyn std::error::Error>>
{
    screen.mv( 35, 0 )?;
    screen.clrtoeol()?;

    if let Some(item_rc) = menu.current_item() {
        let item = item_rc.borrow();
        screen.mvaddstr(
            35,
            2,
            &format!("Current: {} - {}", item.name(), item.description()),
        )?;
    }

    screen.refresh()?;
    Ok(())
}

const Y_AFFICHAGE: i32 = 20;
const X_AFFICHAGE: i32 = 0;
const Y_SAISIE: i32 = 17;
const X_SAISIE: i32 = 0;

fn execution ( screen: &mut Screen, item: &menu::MenuItem, pool: Pool ) -> std::result::Result<(), Box<dyn std::error::Error>>
{
    // variables

    let mut conn = pool.get_conn()?;

    let mut nom: String;
    let mut prenom: String;
    let mut naissance: String;
    let mut rows: Vec<Row>;
    let mut id_nom: u64 = 0;
    let mut id_prenom: u64 = 0;
    let mut id_naissance: u64 = 0;
    let mut commit: bool;

//    let mut x = X_AFFICHAGE;
    for y in Y_AFFICHAGE..=Y_AFFICHAGE + 20
    {
        screen.mv(y, X_AFFICHAGE)?;
        screen.clrtoeol()?;
    }
    screen.refresh()?;

    let mut y = Y_AFFICHAGE;
    if item.name() == "Afficher tous"
    {
        rows = conn.query("SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance
                                         WHERE nom=id_nom AND prenom=id_prenom AND naissance=id_naissance")?;
        for row in rows
        {
            let nom: String = row.get("lib_nom").expect("Colonne nom manquante");
            let prenom: String = row.get("lib_prenom").expect("Colonne prenom manquante");
            let naissance: String = row.get("lib_naissance").expect("Colonne age manquante");
            screen.mvaddstr(y, 2, &format!("{}", nom))?;
            screen.mvaddstr(y, 33, &format!("{}", prenom))?;
            screen.mvaddstr(y, 64, &format!("{}", naissance))?;
            screen.refresh()?;
            y += 1;
        }
    }

    if item.name() == "Afficher un individu"
    {
        for y in Y_SAISIE..=Y_SAISIE + 20
        {
            screen.mv(y, X_SAISIE)?;
            screen.clrtoeol()?;
        }
        screen.refresh()?;

        screen.curs_set ( 1 )?;		// visible
		screen.mvaddstr ( Y_SAISIE, X_SAISIE + 2, "Nom : " )?;
		screen.refresh()?;
		screen.echo ()?;
		nom = screen.getstr ( 30 )?;
		screen.noecho ()?;
		screen.curs_set ( 0 )?;		// invisible

        for y in Y_AFFICHAGE..=Y_AFFICHAGE + 20
        {
            screen.mv(y, X_AFFICHAGE)?;
            screen.clrtoeol()?;
        }
        screen.refresh()?;
        y = Y_AFFICHAGE;
    
        let requete = String::from ( format!( "SELECT lib_nom, lib_prenom, lib_naissance FROM individu, nom, prenom, naissance
                                         WHERE nom=id_nom AND prenom=id_prenom AND naissance=id_naissance AND lib_nom LIKE '{nom}'" ) );
        rows = conn.query ( requete )?;
        if rows.len() > 0
        {
            for row in rows
            {
                nom = row.get("lib_nom").expect("Colonne nom manquante");
                prenom = row.get("lib_prenom").expect("Colonne prenom manquante");
                naissance = row.get("lib_naissance").expect("Colonne age manquante");

                screen.mvaddstr(y, X_AFFICHAGE + 2, &format!("{}", nom))?;
                screen.mvaddstr(y, X_AFFICHAGE + 33, &format!("{}", prenom))?;
                screen.mvaddstr(y, X_AFFICHAGE + 64, &format!("{}", naissance))?;
                screen.refresh()?;
                y += 1;
            }
        }
        else
        {
            screen.mvaddstr ( y, X_AFFICHAGE + 2, &format!("{}", "Pas de résultat." ) )?;
            screen.refresh()?;
        }
    }

    if item.name() == "Ajouter un individu"
    {
        for y in Y_SAISIE..=Y_SAISIE + 20
        {
            screen.mv(y, X_SAISIE)?;
            screen.clrtoeol()?;
        }
        screen.refresh()?;
        screen.curs_set ( 1 )?;		// visible
        screen.echo ()?;

		loop
        {
            screen.mvaddstr ( Y_SAISIE, X_SAISIE + 2, "Nom : " )?;
		    screen.refresh()?;
		    nom = screen.getstr ( 30 )?;
		    screen.mvaddstr ( Y_SAISIE + 1, X_SAISIE + 2, "Prénom : " )?;
		    screen.refresh()?;
		    prenom = screen.getstr ( 30 )?;
		    screen.mvaddstr ( Y_SAISIE + 2, X_SAISIE + 2, "Année de naissance : " )?;
		    screen.refresh()?;
		    naissance = screen.getstr ( 4 )?;
            if nom.len() > 2 && nom.len() <= 30 && prenom.len() > 2 && prenom.len() <= 30 && naissance.len() == 4
            {
                break
            }
        }
        
        screen.noecho ()?;
		screen.curs_set ( 0 )?;		// invisible

        for y in Y_AFFICHAGE..=Y_AFFICHAGE + 20
        {
            screen.mv(y, X_AFFICHAGE)?;
            screen.clrtoeol()?;
        }
        screen.refresh()?;
        y = Y_AFFICHAGE;

        let mut requete = String::from ( format! ( "SELECT * FROM individu, nom, prenom, naissance WHERE nom=id_nom
                                                 AND prenom=id_prenom AND naissance=id_naissance AND lib_nom LIKE '{nom}'
                                                 AND lib_prenom LIKE '{prenom}' AND lib_naissance LIKE '{naissance}'" ) );
        rows = conn.query ( requete )?;
        if rows.len() > 0
        {
            screen.mvaddstr ( y, X_AFFICHAGE, "L'état civil saisi est déjà dans la base de données." )?;
            screen.refresh ()?;
		}
		else				// insertion
		{
            // traitement du nom
            commit = true;
            let mut tx = conn.start_transaction (TxOpts::default())?;
            requete = String::from ( format! ( "SELECT id_nom FROM nom WHERE lib_nom LIKE '{nom}'" ) );
            rows = tx.query ( requete)?;
            if rows.len() > 0
            {
                for row in rows
                {
                    id_nom = row.get ( "id_nom" ).expect ( "Colonne nom manquante" );
                }
            }
            else
            {
                tx.exec_drop ( "INSERT INTO nom ( lib_nom ) VALUES (?)", (&nom, ) )?;
                id_nom = tx.last_insert_id().unwrap();
                if id_nom == 0
                {
                    commit = false
                }
            }
            // traitement du prénom
            requete = String::from ( format! ( "SELECT id_prenom FROM prenom WHERE lib_prenom LIKE '{prenom}'" ) );
            rows = tx.query ( requete)?;
            if rows.len() > 0
            {
                for row in rows
                {
                    id_prenom = row.get ( "id_prenom" ).expect ( "Colonne nom manquante" );
                }
            }
            else
            {
                tx.exec_drop ( "INSERT INTO prenom ( lib_prenom ) VALUES (?)", (&prenom, ) )?;
                id_prenom = tx.last_insert_id().unwrap();
                if id_prenom == 0 
                {
                    commit = false
                }
            }
            // traitement de l'année de naissance
            requete = String::from ( format! ( "SELECT id_naissance FROM naissance WHERE lib_naissance LIKE '{naissance}'" ) );
            rows = tx.query ( requete)?;
            if rows.len() > 0
            {
                for row in rows
                {
                    id_naissance = row.get ( "id_naissance" ).expect ( "Colonne nom manquante" );
                }
            }
            else
            {
                tx.exec_drop ( "INSERT INTO naissance ( lib_naissance ) VALUES (?)", (&naissance, ) )?;
                id_naissance = tx.last_insert_id().unwrap();
                if id_naissance == 0 
                {
                    commit = false
                }
            }
            // update table individu
            tx.exec_drop ( "INSERT INTO individu (nom, prenom, naissance) VALUES ( ?, ?, ? )", ( id_nom, id_prenom, id_naissance ) )?;
            // clore la transaction
            if commit
            {
                tx.commit()?;
            }
            else
            {
                tx.rollback()?;
            }
        }
    }
    
    if item.name() == "Supprimer un individu"
    {
        screen.curs_set ( 1 )?;		// visible
		screen.mvaddstr ( Y_SAISIE, X_SAISIE + 2, "Nom : " )?;
		screen.refresh()?;
		screen.echo ()?;
		nom = screen.getstr ( 30 )?;
		screen.mvaddstr ( Y_SAISIE + 1, X_SAISIE + 2, "Prénom : " )?;
		screen.refresh()?;
		prenom = screen.getstr ( 30 )?;
		screen.mvaddstr ( Y_SAISIE + 2, X_SAISIE + 2, "Année de naissance : " )?;
		screen.refresh()?;
		naissance = screen.getstr ( 4 )?;
        screen.noecho ()?;
		screen.curs_set ( 0 )?;		// invisible
        conn.exec_drop ( "DELETE FROM individu WHERE nom IN (SELECT id_nom FROM nom WHERE lib_nom LIKE ?) AND 
                                prenom IN (SELECT id_prenom FROM prenom WHERE lib_prenom LIKE ?) AND 
                                naissance IN (SELECT id_naissance FROM naissance WHERE lib_naissance LIKE ?)",
                                ( &nom, &prenom, &naissance ) )?;
    }

    if item.name() == "Reset"
    {
        conn.query_drop ( r"DROP TABLE IF EXISTS individu;
                                  CREATE TABLE individu ( nom INT NOT NULL, prenom INT NOT NULL, naissance INT NOT NULL )" )?;
        conn.query_drop ( r"DROP TABLE IF EXISTS naissance;
                                  CREATE TABLE naissance ( id_naissance INT NOT NULL AUTO_INCREMENT, lib_naissance CHAR(4) NOT NULL, PRIMARY KEY ( id_naissance ) )" )?;
        conn.query_drop ( r"DROP TABLE IF EXISTS nom;
                                  CREATE TABLE nom ( id_nom INT NOT NULL AUTO_INCREMENT, lib_nom VARCHAR(30) DEFAULT NULL, PRIMARY KEY ( id_nom ) )" )?;
        conn.query_drop ( r"DROP TABLE IF EXISTS prenom;
                                  CREATE TABLE prenom ( id_prenom INT NOT NULL AUTO_INCREMENT, lib_prenom VARCHAR(30) DEFAULT NULL, PRIMARY KEY ( id_prenom ) )" )?;
    }

    Ok(())
}
